use crate::config::config;
use crate::notify_server::application;
use crate::notify_server::controller::backstage_notify::{BackstageNotifyServer, StreamInfo};
use crate::notify_server::model;
use crate::{entity, enums, repository};
use crate::{helper, mq_manager};
use kgs_err::models::status::Status as KgsStatus;
use kgs_tracing::{tracing, warn};
use protos::backstage_notify::AllReadRequest;
use protos::backstage_notify::{self, *};
use sea_orm::ActiveValue::NotSet;
use sea_orm::{ActiveModelTrait, Set};
use tokio::sync::mpsc::Receiver;

const NOTIFY_PAGE_SIZE: u64 = 10;

/// for後台使用者自己的通知
impl BackstageNotifyServer {
    #[tracing::instrument(skip_all)]
    pub async fn create_connection(
        &self,
        request: ConnectionRequest,
    ) -> Result<Receiver<Result<backstage_notify::Receiver, tonic::Status>>, KgsStatus> {
        // create channel
        let (tx, rx) = tokio::sync::mpsc::channel(1);

        // get user_profile
        let user_account =
            application::user_rpc::get_account_by_user_id(request.client_id, request.user_id)
                .await?;

        // create backstage user
        let user = model::User {
            client_id: request.client_id,
            user_id: request.user_id,
        };

        // insert connection
        {
            let mut connections = self.connections.write().await;
            connections.insert(
                user,
                StreamInfo {
                    user_account: user_account.account,
                    role_ids: request.role_ids,
                    tx,
                },
            );
        }

        Ok(rx)
    }

    #[tracing::instrument]
    pub async fn close_connection(&self, request: CloseRequest) -> Result<(), KgsStatus> {
        // create backstage user
        let user = model::User {
            client_id: request.client_id,
            user_id: request.user_id,
        };

        // remove connection
        {
            let mut connections = self.connections.write().await;
            if let Some(stream) = connections.remove(&user) {
                drop(stream);
            }
        }

        Ok(())
    }

    #[tracing::instrument]
    pub async fn system_broadcast_to_backstage_user(
        &self,
        request: SendRequest,
    ) -> Result<(), KgsStatus> {
        let db = database_manager::sea_orm::get_db();

        // get admin client_id
        let backstage_client_id =
            application::oauth_rpc::get_backstage_client(request.initiator_client_id).await?;

        // check notify_event if backstage_event
        let notify_event = enums::NotifyEvent::from_backstage_proto(request.notify_event())?;
        if !application::notify_handler::is_platform_has_event(
            &*db,
            backstage_client_id,
            &notify_event,
            enums::Platform::Backstage,
        )
        .await?
        {
            return Err(KgsStatus::InvalidArgument);
        }

        // get initiator user_profile
        let user_profile: protos::player::GetUserProfileResponse =
            application::user_rpc::get_user_profile(
                request.initiator_client_id,
                request.initiator_user_id,
            )
            .await?;

        // get client_notify_template_entity
        let client_notify_template_entity = repository::client_notify_template::find_one_by_client_id_and_notify_event_and_notify_type_and_language(
            &*db,
            backstage_client_id,
            notify_event.to_id(),
            enums::NotifyType::InApp,
            &enums::Language::Jp,
        ).await?;

        // replace template with user_profile and key_map
        let (title, content) = application::notify_handler::replace_title_and_content(
            &client_notify_template_entity.title,
            &client_notify_template_entity.content,
            &user_profile,
            &request.key_map,
        );

        tokio::try_join!(
            // 處理本機上的用戶通知
            self.handle_notify(
                backstage_client_id,
                &request.role_ids,
                enums::NotifyType::InApp,
                client_notify_template_entity.client_notify_event,
                &title,
                &content,
            ),
            // 轉發通知到其他pod
            Self::forward_notify(backstage_notify::ForwardNotifyRequest {
                client_id: backstage_client_id,
                role_ids: request.role_ids.clone(),
                client_notify_event_id: client_notify_template_entity.client_notify_event,
                title: title.clone(),
                content: content.clone(),
            }),
        )?;

        Ok(())
    }

    pub async fn handle_notify(
        &self,
        backstage_client_id: i64,
        role_ids: &Vec<i64>,
        notify_type: enums::NotifyType,
        client_notify_event: i64,
        title: &str,
        content: &str,
    ) -> Result<(), KgsStatus> {
        // define internal function
        #[tracing::instrument]
        async fn send_to_local_user(
            notify_id: i64,
            stream: &StreamInfo,
            title: &str,
            content: &str,
        ) -> Result<(), KgsStatus> {
            // create response
            let response = backstage_notify::receiver::Message::Notify(backstage_notify::Notify {
                notify_id: notify_id,
                notify_level: enums::NotifyLevel::System.to_id(),
                title: title.to_string(),
                content: content.to_string(),
                create_at: chrono::Utc::now().timestamp_millis(),
                notify_status: enums::NotifyStatus::Unread.to_id(),
            });

            // send message
            stream
                .tx
                .clone()
                .send(Ok(backstage_notify::Receiver {
                    message: Some(response),
                }))
                .await
                .map_err(|e| {
                    warn!("send message failed: {:?}", e);
                    KgsStatus::UserConnectionNotFound
                })?;

            Ok(())
        }

        // get db transaction
        let txn = database_manager::sea_orm::get_trans().await.map_err(|e| {
            warn!("get db failed: {:?}", e);
            KgsStatus::InternalServerError
        })?;

        // broadcast to backstage user
        let connections = self.connections.read().await;
        for (user, stream_info) in connections.iter() {
            // 觸發事件的使用者的backstage 與 後台人員的client_id相同
            if backstage_client_id != user.client_id {
                continue;
            }

            // 後台人員擁有該權限才會收到廣播
            if !application::notify_handler::has_common_role(role_ids, &stream_info.role_ids) {
                continue;
            }

            // create notify_id
            let notify_id = helper::generate_snowflake_id().await;

            // send message
            if let Err(e) = send_to_local_user(notify_id, stream_info, &title, &content).await {
                warn!("send message failed: {:?}", e);
                {
                    self.connections.write().await.remove(user);
                }
                continue;
            }

            // insert notify_record
            let _notify_record_entity = entity::notify_record::ActiveModel {
                id: Set(notify_id),
                client_id: Set(user.client_id),
                user_id: Set(user.user_id),
                user_account: Set(stream_info.user_account.clone()),
                client_notify_event_id: Set(client_notify_event),
                sender_id: Set(0),
                sender_account: Set("System".to_string()),
                sender_ip: Set(None),
                notify_type: Set(notify_type),
                notify_level: Set(enums::NotifyLevel::System),
                notify_status: Set(enums::NotifyStatus::Unread),
                title: Set(title.to_string()),
                content: Set(content.to_string()),
                create_at: NotSet,
                update_at: NotSet,
            }
            .insert(&txn)
            .await
            .map_err(|e| {
                warn!("insert notify_record failed: {:?}", e);
                KgsStatus::InternalServerError
            })?;
        }

        // commit
        txn.commit().await.map_err(|e| {
            warn!("commit failed: {:?}", e);
            KgsStatus::InternalServerError
        })?;

        Ok(())
    }

    // 轉發notify到其他pod
    async fn forward_notify(
        request: backstage_notify::ForwardNotifyRequest,
    ) -> Result<(), KgsStatus> {
        // define internal function
        async fn send(
            ip: String,
            request: backstage_notify::ForwardNotifyRequest,
        ) -> Result<(), KgsStatus> {
            let dst = format!("http://{}:{}", ip, config::get_host().service_port);

            // get client
            let mut client =
                back_stage_notify_service_client::BackStageNotifyServiceClient::connect(dst)
                    .await
                    .map_err(|e| {
                        warn!("connect to other pod failed: {:?}", e);
                        KgsStatus::InternalServerError
                    })?;

            // forward notify
            let _res = client.forward_notify(request).await.map_err(|e| {
                warn!("forward notify failed: {:?}", e);
                kgs_err::models::status::tonic_to_kgs(e)
            })?;

            Ok(())
        }

        // get other pod ips
        let ip_list = application::kube_api::get_other_pod_ips().await;

        let mut futures = vec![];
        for ip in ip_list {
            futures.push(send(ip, request.clone()));
        }

        // wait all futures
        futures::future::try_join_all(futures).await?;

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
                if status == enums::NotifyStatus::Delete as i32 {
                    Err(KgsStatus::InvalidArgument)
                } else {
                    Ok(enums::NotifyStatus::try_from(status)?)
                }
            })
            .transpose()?;

        // transpose notify_level
        let notify_level = request
            .notify_level
            .map(|level| enums::NotifyLevel::try_from(level))
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
            enums::NotifyStatus::try_from(request.notify_status)?,
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

    pub async fn get_notify_by_id(
        request: backstage_notify::GetNotifyByIdRequest,
    ) -> Result<Notify, KgsStatus> {
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
}

/// for 後台管理通知相關api
impl BackstageNotifyServer {
    #[tracing::instrument]
    pub async fn get_user_notify_records(
        mut request: backstage_notify::GetUserNotifyRecordRequest,
    ) -> Result<backstage_notify::GetUserNotifyRecordResponse, KgsStatus> {
        let db = database_manager::sea_orm::get_db();

        let frontend_client_id =
            application::oauth_rpc::get_frontend_client(request.client_id).await?;
        request.client_id = frontend_client_id;

        let page_size = request.page_size.unwrap_or(NOTIFY_PAGE_SIZE).max(1);
        let now_page = request.now_page.unwrap_or(1).max(1);

        let (entities, total_rows, total_page) =
            repository::notify_record::get_user_notify_records_for_backstage(
                &*db, page_size, now_page, request,
            )
            .await?;

        Ok(backstage_notify::GetUserNotifyRecordResponse {
            list: entities
                .into_iter()
                .map(|entity| entity.to_proto())
                .collect(),
            total_rows,
            total_page,
            now_page: now_page,
        })
    }

    #[tracing::instrument]
    pub async fn backstage_send_to_user(
        request: backstage_notify::BackstageSendToUserRequest,
    ) -> Result<backstage_notify::Empty, KgsStatus> {
        // get transaction
        let txn = database_manager::sea_orm::get_trans().await.map_err(|e| {
            warn!("get db failed: {:?}", e);
            KgsStatus::InternalServerError
        })?;

        // get frontend_client_id
        let frontend_client_id =
            application::oauth_rpc::get_frontend_client(request.client_id).await?;

        // get sender's user profile
        let sender_account =
            application::user_rpc::get_account_by_user_id(request.client_id, request.sender_id)
                .await?;

        // transpose notify_level
        let notify_level = enums::NotifyLevel::try_from(request.notify_level)?;

        // get receiver_account
        let receivers = if request.is_all {
            application::user_rpc::get_accounts_by_client_id(frontend_client_id)
                .await?
                .user_accounts
        } else if request.receiver_ids.len() > 0 {
            application::user_rpc::get_accounts_by_user_ids(
                frontend_client_id,
                request.receiver_ids,
            )
            .await?
            .user_accounts
        } else if request.vip_levels.len() > 0 {
            application::user_rpc::get_accounts_by_vip_level(frontend_client_id, request.vip_levels)
                .await?
                .user_accounts
        } else {
            return Err(KgsStatus::InvalidArgument);
        };

        // if need to save the event
        if request.is_save_as_event && request.client_event_name.is_some() {
            Self::create_custom_client_event(
                &txn,
                frontend_client_id,
                sender_account.account.clone(),
                request.client_event_name.unwrap_or_default(),
                request.client_event_memo.unwrap_or_default(),
                &enums::Language::Jp,
                &request.templates,
            )
            .await?;
        }

        // insert a new backstage_send_task
        let task_id = helper::generate_snowflake_id().await;
        let task_entity = entity::backstage_send_task::ActiveModel {
            id: Set(task_id),
            client_event_id: Set(request.client_event_id.unwrap_or_default()),
            client_id: Set(request.client_id),
            sender_id: Set(request.sender_id),
            sender_ip: Set(request.sender_ip),
            sender_account: Set(sender_account.account),
            task_name: Set(Self::get_task_name(&request.templates)),
            notify_level: Set(notify_level),
            task_status: Set(enums::TaskStatus::Pending),
            receiver_count: Set(receivers.len() as i32),
            receiver_account: Set(receivers
                .iter()
                .map(|account| account.account.to_owned())
                .collect()),
            receiver_id: Set(receivers.iter().map(|account| account.user_id).collect()),
            create_at: NotSet,
            update_at: NotSet,
            error_message: NotSet,
        }
        .insert(&txn)
        .await
        .map_err(|e| {
            warn!("insert backstage_send_task failed: {:?}", e);
            KgsStatus::InternalServerError
        })?;

        // insert backstage_send_task_details
        let mut templates_models = vec![];
        for template in request.templates {
            if template.title.len() > 0 && template.content.len() > 0 {
                let entity = repository::backstage_send_task_detail::insert(
                    &txn,
                    task_id,
                    enums::NotifyType::try_from(template.notify_type)?,
                    notify_level,
                    template.title,
                    template.content,
                )
                .await?;
                templates_models.push(entity.to_publish_model());
            }
        }

        // push to notify_queue
        let mq_task = task_entity.to_publish_model(frontend_client_id, templates_models);
        mq_manager::publish_batch_notify(&mq_task)
            .await
            .map_err(|e| {
                warn!("publish batch notify failed: {:?}", e);
                KgsStatus::InternalServerError
            })?;

        // commit
        txn.commit().await.map_err(|e| {
            warn!("commit failed: {:?}", e);
            KgsStatus::InternalServerError
        })?;

        Ok(backstage_notify::Empty {})
    }

    #[tracing::instrument]
    pub async fn get_client_event_summary(
        request: backstage_notify::GetClientEventSummaryRequest,
    ) -> Result<backstage_notify::ClientEventSummaryList, KgsStatus> {
        let db = database_manager::sea_orm::get_db();

        let frontend_client_id =
            application::oauth_rpc::get_frontend_client(request.client_id).await?;

        let entities =
            repository::client_notify_event::get_list_by_client_id_and_is_system_and_platform(
                &*db,
                frontend_client_id,
                enums::Platform::Frontend, // 只獲取前台的事件
                request.is_system,
            )
            .await?;

        Ok(backstage_notify::ClientEventSummaryList {
            list: entities
                .into_iter()
                .map(|entity| entity.to_proto())
                .collect(),
        })
    }

    #[tracing::instrument]
    pub async fn get_client_templates(
        request: backstage_notify::GetClientTemplatesRequest,
    ) -> Result<backstage_notify::ClientTemplateList, KgsStatus> {
        let db = database_manager::sea_orm::get_db();

        let front_client_id =
            application::oauth_rpc::get_frontend_client(request.client_id).await?;
        let entities = repository::client_notify_template::find_list_by_client_id_and_notify_event(
            &*db,
            front_client_id,
            request.client_event_id,
            &enums::Language::Jp,
        )
        .await?;

        Ok(backstage_notify::ClientTemplateList {
            list: entities
                .into_iter()
                .map(|entity| entity.to_proto())
                .collect(),
        })
    }

    async fn create_custom_client_event<C>(
        db: &C,
        client_id: i64,
        editor_account: String,
        name: String,
        memo: String,
        language: &enums::Language,
        templates: &Vec<backstage_notify::Template>,
    ) -> Result<(), KgsStatus>
    where
        C: sea_orm::ConnectionTrait,
    {
        // create client_notify_event
        let event_entity = repository::client_notify_event::create_custom_event(
            db,
            client_id,
            editor_account,
            name,
            memo,
        )
        .await?;

        // create client_notify_template
        repository::client_notify_template::create_custom_templates(
            db,
            client_id,
            event_entity.id,
            language,
            &templates,
        )
        .await?;

        Ok(())
    }

    pub async fn get_notify_task_list(
        request: backstage_notify::GetNotifyTaskListRequest,
    ) -> Result<backstage_notify::NotifyTaskList, KgsStatus> {
        let db = database_manager::sea_orm::get_db();

        let page_size = request.page_size.unwrap_or(NOTIFY_PAGE_SIZE).max(1);
        let now_page = request.now_page.unwrap_or(1).max(1);

        let (entities, total_rows, total_page) =
            repository::backstage_send_task::get_notify_task_list(
                &*db, page_size, now_page, request,
            )
            .await?;

        Ok(backstage_notify::NotifyTaskList {
            list: entities
                .into_iter()
                .map(|entity| entity.to_proto())
                .collect(),
            total_rows,
            total_page,
            now_page: now_page,
        })
    }

    pub async fn get_notify_task_details(
        request: backstage_notify::GetNotifyTaskDetailsRequest,
    ) -> Result<backstage_notify::NotifyTaskDetailList, KgsStatus> {
        let db = database_manager::sea_orm::get_db();

        let details =
            repository::backstage_send_task_detail::find_list_by_task_id(&*db, request.task_id)
                .await?;

        Ok(NotifyTaskDetailList {
            list: details
                .into_iter()
                .map(|entity| entity.to_proto())
                .collect(),
        })
    }

    pub async fn get_client_event(
        mut request: backstage_notify::GetClientEventRequest,
    ) -> Result<backstage_notify::ClientEventList, KgsStatus> {
        let db = database_manager::sea_orm::get_db();

        let frontend_client_id =
            application::oauth_rpc::get_frontend_client(request.client_id).await?;
        request.client_id = frontend_client_id;

        let page_size = request.page_size.unwrap_or(NOTIFY_PAGE_SIZE).max(1);
        let now_page = request.now_page.unwrap_or(1).max(1);

        let (records, total_rows, total_page) =
            repository::client_notify_event::get_list_by_get_client_event_request_proto(
                &*db, page_size, now_page, request,
            )
            .await?;

        Ok(backstage_notify::ClientEventList {
            list: records.into_iter().map(|model| model.to_proto()).collect(),
            now_page,
            total_rows,
            total_page,
        })
    }

    pub async fn update_client_event(
        request: backstage_notify::UpdateClientEventRequest,
    ) -> Result<backstage_notify::Empty, KgsStatus> {
        let txn = database_manager::sea_orm::get_trans().await.map_err(|e| {
            warn!("get db failed: {:?}", e);
            KgsStatus::InternalServerError
        })?;

        let (frontend_client_id, user_account) = tokio::try_join!(
            application::oauth_rpc::get_frontend_client(request.client_id),
            application::user_rpc::get_account_by_user_id(request.client_id, request.user_id)
        )
        .map_err(|e| {
            warn!("get user_profile or frontend_client_id failed: {:?}", e);
            KgsStatus::InternalServerError
        })?;

        // update client_event
        repository::client_notify_event::update_client_event(
            &txn,
            frontend_client_id,
            request.client_event_id,
            request.event_name,
            request.notify_types,
            request.memo,
            user_account.account,
        )
        .await?;

        // update client_notify_template
        repository::client_notify_template::update_client_templates(
            &txn,
            frontend_client_id,
            request.client_event_id,
            &request.templates,
        )
        .await?;

        txn.commit().await.map_err(|e| {
            warn!("commit failed: {:?}", e);
            KgsStatus::InternalServerError
        })?;

        Ok(Empty {})
    }

    pub async fn delete_client_event(
        request: backstage_notify::DeleteClientEventRequest,
    ) -> Result<backstage_notify::Empty, KgsStatus> {
        let txn = database_manager::sea_orm::get_trans().await.map_err(|e| {
            warn!("get db failed: {:?}", e);
            KgsStatus::InternalServerError
        })?;

        let frontend_client_id =
            application::oauth_rpc::get_frontend_client(request.client_id).await?;

        // delete client_notify_event
        repository::client_notify_event::delete_custom_client_notify_event(
            &txn,
            frontend_client_id,
            request.client_event_id,
        )
        .await?;

        // delete client_notify_template
        repository::client_notify_template::delete_custom_templates(
            &txn,
            frontend_client_id,
            request.client_event_id,
        )
        .await?;

        // commit
        txn.commit().await.map_err(|e| {
            warn!("commit failed: {:?}", e);
            KgsStatus::InternalServerError
        })?;

        Ok(Empty {})
    }

    pub async fn create_client_event(
        request: backstage_notify::CreateClientEventRequest,
    ) -> Result<backstage_notify::Empty, KgsStatus> {
        let txn = database_manager::sea_orm::get_trans().await.map_err(|e| {
            warn!("get db failed: {:?}", e);
            KgsStatus::InternalServerError
        })?;

        let (user_account, frontend_client_id) = tokio::try_join!(
            application::user_rpc::get_account_by_user_id(request.client_id, request.user_id),
            application::oauth_rpc::get_frontend_client(request.client_id)
        )
        .map_err(|e| {
            warn!("get user_profile or frontend_client_id failed: {:?}", e);
            KgsStatus::InternalServerError
        })?;

        // create client_notify_event
        Self::create_custom_client_event(
            &txn,
            frontend_client_id,
            user_account.account,
            request.event_name,
            request.event_memo.unwrap_or_default(),
            &enums::Language::Jp,
            &request.templates,
        )
        .await?;

        // commit
        txn.commit().await.map_err(|e| {
            warn!("commit failed: {:?}", e);
            KgsStatus::InternalServerError
        })?;

        Ok(Empty {})
    }
}

impl BackstageNotifyServer {
    fn get_task_name(templates: &Vec<backstage_notify::Template>) -> String {
        let mut task_name = String::new();
        for template in templates {
            task_name.push_str(&template.title);
            task_name.push_str(" ");
        }
        task_name
    }
}
