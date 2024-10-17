use crate::enums::{NotifyLevel, NotifyStatus};
use crate::notify_server::application;
use crate::notify_server::controller::frontend_notify::FrontendNotifyServer;
use crate::notify_server::model;
use crate::repository;
use crate::{config, helper};
use crate::{enums, mq_manager};
use frontend_notify_service_client::FrontendNotifyServiceClient;
use kgs_err::models::status::Status as KgsStatus;
use kgs_tracing::{tracing, warn};
use protos::frontend_notify::{self, *};
use tokio::sync::mpsc::Receiver;

const NOTIFY_PAGE_SIZE: u64 = 10;

impl FrontendNotifyServer {
    #[tracing::instrument(skip_all)]
    pub async fn create_connection(
        &self,
        request: ConnectionRequest,
    ) -> Result<Receiver<Result<frontend_notify::Receiver, tonic::Status>>, KgsStatus> {
        // create channel
        let (tx, rx) = tokio::sync::mpsc::channel(1); // 先測試 每一個user一個channel 避免爆掉

        // create frontend user
        let frontend_user = model::User {
            client_id: request.client_id,
            user_id: request.user_id,
        };

        // insert connection
        {
            let mut connections = self.connections.write().await;
            connections.insert(frontend_user, tx);
        }

        // save user location server to redis
        application::notify_handler::save_user_located_server_to_redis(request.user_id)?;

        Ok(rx)
    }

    #[tracing::instrument(skip_all)]
    pub async fn close_connection(&self, request: ConnectionRequest) -> Result<(), KgsStatus> {
        // create frontend user
        let frontend_user = model::User {
            client_id: request.client_id,
            user_id: request.user_id,
        };

        // remove connection
        {
            let mut connection = self.connections.write().await;
            if let Some(tx) = connection.remove(&frontend_user) {
                drop(tx);
            }
        }

        // remove user location server from redis
        application::notify_handler::remove_user_located_server_from_redis(request.user_id)?;

        Ok(())
    }

    #[tracing::instrument]
    pub async fn system_to_frontend_user(&self, request: SendRequest) -> Result<(), KgsStatus> {
        let db = database_manager::sea_orm::get_db();

        // check notify_event is frontend event
        let notify_event = enums::NotifyEvent::from_frontend_proto(request.notify_event())?;
        if !application::notify_handler::is_platform_has_event(
            &*db,
            request.client_id,
            &notify_event,
            enums::Platform::Frontend,
        )
        .await?
        {
            return Err(KgsStatus::InvalidArgument);
        }

        // get user_profile
        let user_profile =
            application::user_rpc::get_user_profile(request.client_id, request.user_id).await?;

        // get client_notify_template_entity
        let client_notify_template_entities =
            repository::client_notify_template::find_list_by_client_id_and_notify_event_is_on(
                &*db,
                request.client_id,
                notify_event.to_id(),
                &enums::Language::Jp, // TODO: 目前只有日文,未來要從user_profile取得
            )
            .await?;

        for template in client_notify_template_entities {
            // get receive_address
            let receive_address = application::notify_handler::get_receive_address(
                &template.notify_type,
                &user_profile.email,
                &user_profile.phone,
            );

            // send message
            let notify_id = helper::generate_snowflake_id().await;

            // publish notify
            mq_manager::publish_single_notify(&mq_manager::SingleNotifyModel {
                notify_id,
                client_event_id: template.client_notify_event,
                client_id: request.client_id,
                user_id: request.user_id,
                sender_id: 0, // system 發送為0
                sender_account: "System".to_string(),
                sender_ip: None,
                notify_type: template.notify_type,
                notify_level: enums::NotifyLevel::System,
                title: template.title.clone(),
                content: template.content.clone(),
                receive_address: receive_address.to_string(),
                key_map: request.key_map.clone(),
            })
            .await
            .map_err(|err| {
                warn!("publish_frontend_notify failed: {:?}", err);
                KgsStatus::InternalServerError
            })?;
        }

        Ok(())
    }

    #[tracing::instrument]
    pub async fn send_message_in_app(
        &self,
        client_id: i64,
        user_id: i64,
        notify_id: i64,
        notify_level: enums::NotifyLevel,
        title: &str,
        content: &str,
    ) -> Result<(), KgsStatus> {
        // get user location server from redis
        if let Some(pod_ip) =
            application::notify_handler::get_user_located_server_from_redis(user_id)?
        {
            // if the user located in the same server handle the notify
            // else send the notify to the user located server
            if config::config::get_kubernetes().pod_ip == pod_ip {
                self.handle_notify(client_id, user_id, notify_id, notify_level, title, content)
                    .await?;
            } else {
                self.forward_notify(
                    &pod_ip,
                    client_id,
                    user_id,
                    notify_id,
                    notify_level,
                    title,
                    content,
                )
                .await?;
            }
        }

        Ok(())
    }

    #[tracing::instrument]
    pub async fn handle_notify(
        &self,
        client_id: i64,
        user_id: i64,
        notify_id: i64,
        notify_level: enums::NotifyLevel,
        title: &str,
        content: &str,
    ) -> Result<(), KgsStatus> {
        let result = (|| async {
            let tx: tokio::sync::mpsc::Sender<Result<frontend_notify::Receiver, tonic::Status>> =
                self.connections
                    .read()
                    .await
                    .get(&model::User {
                        client_id: client_id,
                        user_id: user_id,
                    })
                    .ok_or(KgsStatus::UserConnectionNotFound)?
                    .clone();

            // create_notify_proto
            let response = protos::frontend_notify::receiver::Message::Notify(
                protos::frontend_notify::Notify {
                    notify_id: notify_id,
                    notify_level: notify_level.to_id(),
                    title: title.to_string(),
                    content: content.to_string(),
                    create_at: chrono::Utc::now().timestamp_millis(),
                    notify_status: enums::NotifyStatus::Unread.to_id(),
                },
            );

            // send notify
            tx.send(Ok(protos::frontend_notify::Receiver {
                message: Some(response),
            }))
            .await
            .map_err(|e| {
                warn!("send notify error: {:?}", e);
                KgsStatus::UserConnectionNotFound
            })?;

            Ok::<(), KgsStatus>(())
        })()
        .await;

        // if the user connection not found remove the user located server from redis
        if let Err(err) = result {
            application::notify_handler::remove_user_located_server_from_redis(user_id)?;
            return Err(err);
        }

        Ok(())
    }

    #[tracing::instrument]
    pub async fn get_notify_records(
        request: GetNotifyRecordRequest,
    ) -> Result<GetNotifyRecordResponse, KgsStatus> {
        // transpose notify_status
        let notify_status = request
            .notify_status
            .map(|status| {
                if status == NotifyStatus::Delete as i32 {
                    Err(KgsStatus::InvalidArgument)
                } else {
                    Ok(NotifyStatus::try_from(status)?)
                }
            })
            .transpose()?;

        // transpose notify_level
        let notify_level = request
            .notify_level
            .map(|level| NotifyLevel::try_from(level))
            .transpose()?;

        let now_page = request.now_page.unwrap_or(1).max(1);

        // get db connection
        let conn = database_manager::sea_orm::get_db();

        // get notify_records and unread_count in parallel
        match tokio::try_join!(
            repository::notify_record::find_all_app_record_by_user_id_and_status(
                &*conn,
                request.client_id,
                request.user_id,
                notify_status,
                notify_level.clone(),
                now_page,
                NOTIFY_PAGE_SIZE,
            ),
            repository::notify_record::get_unread_app_record_notify_count(
                &*conn,
                request.client_id,
                request.user_id,
                notify_level
            )
        ) {
            Ok(((record_entities, total_rows, total_page), unread_count)) => {
                Ok(GetNotifyRecordResponse {
                    list: record_entities
                        .into_iter()
                        .map(|record| record.to_proto())
                        .collect(),
                    total_rows,
                    total_page,
                    now_page,
                    unread_count,
                })
            }
            Err(e) => {
                warn!("get notify_records failed: {:?}", e);
                Err(KgsStatus::InternalServerError)
            }
        }
    }

    #[tracing::instrument]
    pub async fn update_notify_records(
        request: UpdateNotifyRecordRequest,
    ) -> Result<UpdateNotifyRecordResponse, KgsStatus> {
        // get db transaction
        let txn = database_manager::sea_orm::get_trans()
            .await
            .map_err(|err| {
                warn!("get db failed: {:?}", err);
                KgsStatus::InternalServerError
            })?;

        // update notify_records
        let result = repository::notify_record::update_notify_records(
            &txn,
            request.client_id,
            request.user_id,
            NotifyStatus::try_from(request.notify_status)?,
            request.notify_ids,
        )
        .await?;

        // commit
        txn.commit().await.map_err(|e| {
            warn!("commit failed: {:?}", e);
            KgsStatus::InternalServerError
        })?;

        Ok(UpdateNotifyRecordResponse {
            list: result.into_iter().map(|record| record.to_proto()).collect(),
        })
    }

    #[tracing::instrument]
    pub async fn get_unread_notify_count(
        request: GetUnreadNotifyCountRequest,
    ) -> Result<GetUnreadNotifyCountResponse, KgsStatus> {
        // get db connection
        let conn = database_manager::sea_orm::get_db();

        // get unread notify count
        let count = repository::notify_record::get_all_unread_app_records_count(
            &*conn,
            request.client_id,
            request.user_id,
        )
        .await?;

        Ok(GetUnreadNotifyCountResponse { total_rows: count })
    }

    #[tracing::instrument]
    pub async fn all_read(request: AllReadRequest) -> Result<Empty, KgsStatus> {
        // get txn
        let txn = database_manager::sea_orm::get_trans().await.map_err(|e| {
            warn!("get db failed: {:?}", e);
            KgsStatus::InternalServerError
        })?;

        // transpose notify_level
        let notify_level = request
            .notify_level
            .map(|level| enums::NotifyLevel::try_from(level))
            .transpose()?;

        // update notify_records
        let _all_entities = repository::notify_record::update_all_with_notify_level(
            &txn,
            request.client_id,
            request.user_id,
            notify_level,
            enums::NotifyStatus::Read,
        )
        .await?;

        // commit
        txn.commit().await.map_err(|e| {
            warn!("commit failed: {:?}", e);
            KgsStatus::InternalServerError
        })?;

        Ok(Empty {})
    }

    #[tracing::instrument]
    pub async fn get_notify_by_id(request: GetNotifyByIdRequest) -> Result<Notify, KgsStatus> {
        let db = database_manager::sea_orm::get_db();

        let entity = repository::notify_record::find_one_by_id_and_client_id_user_id(
            &*db,
            request.notify_id,
            request.client_id,
            request.user_id,
        )
        .await?;

        Ok(entity.to_proto())
    }

    /// 轉發通知
    #[tracing::instrument]
    pub async fn forward_notify(
        &self,
        pod_ip: &str,
        client_id: i64,
        user_id: i64,
        notify_id: i64,
        notify_level: enums::NotifyLevel,
        title: &str,
        content: &str,
    ) -> Result<(), KgsStatus> {
        let port = config::config::get_host().service_port;
        let rpc_url = format!("http://{}:{}", pod_ip, port);

        let mut client = FrontendNotifyServiceClient::connect(rpc_url)
            .await
            .map_err(|err| {
                warn!("Failed to connect to notify server: {:?}", err);
                KgsStatus::InternalServerError
            })?;

        let notify = frontend_notify::Notify {
            notify_id,
            notify_level: notify_level.to_id(),
            title: title.to_string(),
            content: content.to_string(),
            create_at: chrono::Utc::now().timestamp_millis(),
            notify_status: NotifyStatus::Unread as i32,
        };

        let request = tonic::Request::new(ForwardNotifyRequest {
            client_id,
            user_id,
            notify: Some(notify),
        });

        client.forward_notify(request).await.map_err(|err| {
            warn!("Failed to send notify: {:?}", err);
            KgsStatus::InternalServerError
        })?;

        Ok(())
    }

    /// 處理收到的轉發通知
    #[tracing::instrument]
    pub async fn handle_forward_notify(
        &self,
        request: ForwardNotifyRequest,
    ) -> Result<Empty, KgsStatus> {
        // parse the request
        let notify = request.notify.ok_or_else(|| {
            warn!("can't found the notify body");
            KgsStatus::MissingBodyArgument
        })?;

        let notify_level = enums::NotifyLevel::try_from(notify.notify_level).map_err(|err| {
            warn!("can't found the notify level: {:#?}", err);
            KgsStatus::InvalidArgument
        })?;

        // handle the notify
        self.handle_notify(
            request.client_id,
            request.user_id,
            notify.notify_id,
            notify_level,
            &notify.title,
            &notify.content,
        )
        .await
        .map_err(|err| err)?;

        Ok(frontend_notify::Empty {})
    }
}
