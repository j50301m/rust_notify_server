use crate::consumers::consumer::Job;
use crate::consumers::error::JobError;
use crate::consumers::send_request_handler;
use crate::entity;
use crate::enums;
use crate::helper;
use crate::mq_manager;
use crate::mq_manager::SingleNotifyModel;
use crate::notify_server::application::user_rpc;
use crate::notify_server::FRONTEND_NOTIFY_SERVER;
use crate::repository;
use futures::StreamExt;
use kgs_tracing::tracing;
use kgs_tracing::{info, warn};
use lapin::Consumer;
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::NotSet;
use sea_orm::Set;
use std::fmt::Debug;
use std::sync::Arc;
use tonic::async_trait;

use super::error::ConsumerError;

#[derive(Debug)]
pub struct SingleNotifyJob {
    job_name: String,
    rabbit_consumer: Option<Consumer>,
    message: Option<Arc<SingleNotifyModel>>,
}

impl SingleNotifyJob {
    pub fn new(job_name: &str) -> Self {
        SingleNotifyJob {
            job_name: job_name.to_string(),
            rabbit_consumer: None,
            message: None,
        }
    }
}

#[async_trait]
impl Job for SingleNotifyJob {
    fn job_name(&self) -> &str {
        &self.job_name
    }

    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("{} start", self.job_name.as_str());

        // get rabbit_mq consumer
        match mq_manager::consume_single_notify(self.job_name()).await {
            Ok(r) => self.rabbit_consumer = Some(r),
            Err(err) => {
                warn!("獲取rabbit_mq consumer錯誤 {}", err);
                return Err(Box::new(err));
            }
        };

        Ok(())
    }

    #[tracing::instrument]
    async fn update(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("{} update", self.job_name.as_str());
        // reset message
        if self.message.is_some() {
            self.message = None;
        }

        // early return if rabbit_consumer is none
        if self.rabbit_consumer.as_ref().is_none() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "rabbit_consumer is none",
            )));
        }

        // get delivery form rabbit_mq
        let delivery = match self.rabbit_consumer.as_mut().unwrap().next().await {
            Some(r) => r.map_err(|err| {
                warn!("獲取Delivery錯誤 {}", err);
                Box::new(err)
            })?,
            None => {
                return Ok(());
            } // if no message, wait for next message
        };

        // ack first
        mq_manager::rabbit_consumer_ack(&delivery)
            .await
            .map_err(|err| {
                warn!("ack錯誤 {}", err);
                Box::new(err)
            })?;

        // get trans from database_manager
        let trans = database_manager::sea_orm::get_trans()
            .await
            .map_err(|err| {
                warn!("獲取db trans錯誤 {}", err);
                Box::new(err)
            })?;

        // parse ConsumerModelNotify from delivery
        let received_model: SingleNotifyModel = match std::str::from_utf8(&delivery.data) {
            Ok(msg_str) => serde_json::from_str(msg_str).map_err(|err| {
                warn!(" deserialize from binary 錯誤{}", err);
                Box::new(err)
            })?,
            Err(err) => {
                warn!("binary轉成str錯誤 {} ", err);
                return Err(Box::new(err));
            }
        };

        // stone received_model to self.message
        let received_model_arc = Arc::new(received_model);
        self.message = Some(Arc::clone(&received_model_arc));

        // get user_profile
        let user_profile =
            user_rpc::get_user_profile(received_model_arc.client_id, received_model_arc.user_id)
                .await
                .map_err(|_err| {
                    let msg = format!(
                        "獲取user_profile錯誤: client_id: {}, user_id: {}",
                        received_model_arc.client_id, received_model_arc.user_id
                    );
                    warn!("{}", msg);
                    Box::new(JobError::new(msg, None))
                })?;

        // replace the template
        let (title, content) =
            crate::notify_server::application::notify_handler::replace_title_and_content(
                &received_model_arc.title,
                &received_model_arc.content,
                &user_profile,
                &received_model_arc.key_map,
            );

        let notify_id = helper::generate_snowflake_id().await;

        // send message
        match received_model_arc.notify_type {
            enums::NotifyType::Email => {
                send_request_handler::send_email(
                    &title,
                    &content,
                    &received_model_arc.receive_address,
                )
                .await?;
            }
            enums::NotifyType::SMS => {
                send_request_handler::send_sms(&content, &received_model_arc.receive_address)
                    .await?;
            }
            enums::NotifyType::InApp => {
                if let Err(e) = FRONTEND_NOTIFY_SERVER
                    .send_message_in_app(
                        received_model_arc.client_id,
                        received_model_arc.user_id,
                        notify_id,
                        received_model_arc.notify_level,
                        &title,
                        &content,
                    )
                    .await
                {
                    warn!("send_message_in_app錯誤 {}", e);
                }
            }
        }

        // insert notify_record to database
        entity::notify_record::ActiveModel {
            id: Set(notify_id),
            client_id: Set(received_model_arc.client_id),
            client_notify_event_id: Set(received_model_arc.client_event_id),
            user_id: Set(received_model_arc.user_id),
            user_account: Set(user_profile.account),
            sender_id: Set(received_model_arc.sender_id),
            sender_account: Set(received_model_arc.sender_account.clone()),
            sender_ip: Set(received_model_arc.sender_ip.clone()),
            title: Set(title),
            content: Set(content),
            notify_type: Set(received_model_arc.notify_type),
            notify_level: Set(received_model_arc.notify_level),
            notify_status: Set(enums::NotifyStatus::Unread),
            create_at: NotSet,
            update_at: NotSet,
        }
        .insert(&trans)
        .await
        .map_err(|err| {
            warn!("插入records錯誤 {}", err);
            Box::new(err)
        })?;

        // insert success message to database
        repository::mq_success_record::create(
            &trans,
            received_model_arc.notify_id,
            received_model_arc.client_id,
            received_model_arc.user_id,
            received_model_arc.sender_id,
            &received_model_arc.title.clone(),
            &received_model_arc.content.clone(),
            &received_model_arc.notify_type.clone(),
        )
        .await
        .map_err(|err| {
            warn!("插入records錯誤 {}", err);
            Box::new(err)
        })?;

        // commit
        trans.commit().await.map_err(|err| {
            warn!("commit錯誤 {}", err);
            Box::new(err)
        })?;

        Ok(())
    }

    async fn end(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("{} end", self.job_name.as_str());
        Ok(())
    }

    async fn error_handler(
        &mut self,
        err: ConsumerError,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        warn!("{} error: message:{}", self.job_name.as_str(), err);
        match err {
            ConsumerError::StartStateError(err) => {
                warn!("StartStateError {}", err);
                // 如果在start階段發生錯誤 我選擇在重新執行一次 ,也可以直接return err
                self.start().await?;
            }
            ConsumerError::UpdateStateError(err) => {
                warn!("UpdateStateError {}", err);

                // get trans from database_manager
                let trans = database_manager::sea_orm::get_trans()
                    .await
                    .map_err(|err| {
                        warn!("獲取db trans錯誤 {}", err);
                        Box::new(err)
                    })?;

                // get received_model from self.message
                if let Some(received_model_arc) = &self.message {
                    let received_model = Arc::clone(&received_model_arc);

                    // insert failed message to database
                    repository::mq_failed_record::create(
                        &trans,
                        Some(received_model.notify_id),
                        Some(received_model.client_id),
                        Some(received_model.user_id),
                        Some(received_model.sender_id),
                        Some(received_model.title.clone()),
                        Some(received_model.notify_type.clone()),
                        Some(received_model.content.clone()),
                        Some(err.to_string()),
                    )
                    .await
                    .map_err(|err| {
                        warn!("插入records錯誤 {}", err);
                        Box::new(err)
                    })?;

                    // clear self.message
                    self.message = None;

                    // commit
                    trans.commit().await.map_err(|err| {
                        warn!("commit錯誤 {}", err);
                        Box::new(err)
                    })?;
                }
            }
            ConsumerError::EndStateError(err) => {
                warn!("EndStateError {}", err);
            }
        }

        Ok(())
    }

    fn is_continue(&self) -> bool {
        true
    }
}
