use crate::consumers::consumer::Job;

use crate::consumers::error::JobError;
use crate::mq_manager::{BatchNotifyModel, SingleNotifyModel};
use crate::notify_server::application::user_rpc;
use crate::repository;
use crate::{enums, notify_server};
use crate::{helper, mq_manager};
use futures::StreamExt;
use kgs_tracing::{info, warn};
use lapin::Consumer;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tonic::async_trait;

use super::error::ConsumerError;

#[derive(Debug)]
pub struct BatchNotifyJob {
    job_name: String,
    rabbit_consumer: Option<Consumer>,
    message: Option<Arc<BatchNotifyModel>>,
}

impl BatchNotifyJob {
    pub fn new(job_name: &str) -> Self {
        BatchNotifyJob {
            job_name: job_name.to_string(),
            rabbit_consumer: None,
            message: None,
        }
    }
}

#[async_trait]
impl Job for BatchNotifyJob {
    fn job_name(&self) -> &str {
        &self.job_name
    }

    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("{} start", self.job_name.as_str());

        // get rabbit_mq consumer
        match mq_manager::consume_batch_notify(self.job_name()).await {
            Ok(r) => self.rabbit_consumer = Some(r),
            Err(err) => {
                warn!("獲取rabbit_mq consumer錯誤 {}", err);
                return Err(Box::new(err));
            }
        };

        Ok(())
    }

    async fn update(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("{} update", self.job_name.as_str());
        // reset message
        if self.message.is_some() {
            self.message = None;
        }

        // early return if rabbit_consumer is none
        if self.rabbit_consumer.as_ref().is_none() {
            return Err(Box::new(JobError::new(
                "rabbit_consumer is none".to_string(),
                None,
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

        // ack
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
        let batch_notify_model: BatchNotifyModel = match std::str::from_utf8(&delivery.data) {
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
        let batch_notify_model_arc = Arc::new(batch_notify_model);
        self.message = Some(Arc::clone(&batch_notify_model_arc));

        // publish each message to user
        publish_each_message_to_user(batch_notify_model_arc.clone()).await?;

        // update task status
        repository::backstage_send_task::update_task_status(
            &trans,
            batch_notify_model_arc.task_id,
            enums::TaskStatus::Success,
            None,
        )
        .await
        .map_err(|err| {
            warn!("update task status錯誤 {}", err);
            Box::new(JobError::new("更新task status時發生錯誤".to_string(), None))
        })?;

        // commit
        trans.commit().await.map_err(|err| {
            warn!("commit trans錯誤 {}", err);
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
        warn!("{} error_handler {}", self.job_name.as_str(), err);

        match err {
            ConsumerError::StartStateError(err) => {
                warn!("StartStateError {}", err);
                // 如果在start階段發生錯誤 我選擇在重新執行一次 ,也可以直接return err
                self.start().await?;
            }
            ConsumerError::UpdateStateError(err) => {
                // 如果在update階段發生錯誤 我會檢查是否有拿到message 如果有 將 task狀態改回fail 並記錄 err message
                // 然後繼續下一個任務
                warn!("UpdateStateError {}", err);

                // get trans from database_manager
                let trans = database_manager::sea_orm::get_trans()
                    .await
                    .map_err(|err| {
                        warn!("獲取db trans錯誤 {}", err);
                        Box::new(err)
                    })?;

                if let Some(model_arc) = &self.message {
                    repository::backstage_send_task::update_task_status(
                        &trans,
                        model_arc.task_id,
                        enums::TaskStatus::Fail,
                        Some(err.to_string()),
                    )
                    .await
                    .map_err(|err| {
                        warn!("update task status錯誤 {}", err);
                        Box::new(JobError::new("更新task status時發生錯誤".to_string(), None))
                    })?;

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

async fn publish_each_message_to_user(
    model: Arc<BatchNotifyModel>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // get each user address
    let user_addresses =
        user_rpc::get_email_and_phone_by_user_ids(model.frontend_client_id, &model.receiver_ids)
            .await
            .map_err(|_err| {
                let msg: String = format!("獲取user address錯誤 client_id: {}", model.client_id);
                warn!("{}", msg);
                Box::new(JobError::new(msg, None))
            })?;

    // publish each message
    for user_address in user_addresses.email_and_phone.iter() {
        // send each template to user
        for template in &model.templates {
            let notify_id = helper::generate_snowflake_id().await;
            let receive_address =
                notify_server::application::notify_handler::get_receive_address_opt(
                    &template.notify_type,
                    &user_address.email,
                    &user_address.phone,
                );
            let single_model = SingleNotifyModel {
                notify_id,
                client_event_id: model.client_event_id,
                user_id: user_address.user_id,
                sender_id: model.sender_id,
                sender_account: model.sender_account.clone(),
                sender_ip: model.sender_ip.clone(),
                client_id: model.frontend_client_id,
                notify_type: template.notify_type,
                notify_level: template.notify_level,
                title: template.title.clone(),
                content: template.content.clone(),
                receive_address,
                key_map: HashMap::new(),
            };

            // publish message
            mq_manager::publish_single_notify(&single_model).await?;
        }
    }

    Ok(())
}
