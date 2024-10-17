use kgs_tracing::{error, info, warn};
use std::sync::Arc;
use tokio::{sync::Mutex, task::JoinHandle};
use tonic::async_trait;

use super::error::ConsumerError;

#[async_trait]
pub trait Job: Send + Sync + 'static {
    fn job_name(&self) -> &str;
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn update(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn end(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn error_handler(
        &mut self,
        err: ConsumerError,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn is_continue(&self) -> bool;
}

pub struct Consumer {
    job: Box<dyn Job>,
    job_retry_times: i32,
}

impl Consumer {
    pub fn new<T: Job>(job: T, job_retry_times: i32) -> Self {
        Self {
            job: Box::new(job),
            job_retry_times,
        }
    }

    pub async fn execute(self) {
        let self_move = Arc::new(Mutex::new(self));
        let mut retries = { self_move.lock().await.job_retry_times };

        while retries > 0 || retries == -1 {
            let self_clone = Arc::clone(&self_move);
            let task: JoinHandle<Result<(), ConsumerError>> = tokio::spawn(async move {
                let mut consumer = self_clone.lock().await;

                // start job
                if let Err(err) = consumer.job.start().await {
                    let err = if let Err(handler_err) = consumer
                        .job
                        .error_handler(ConsumerError::StartStateError(err))
                        .await
                    {
                        warn!("{} error: message:{}", consumer.job.job_name(), handler_err);
                        Some(handler_err)
                    } else {
                        None
                    };

                    // if error handler failed return break the thread
                    if let Some(err) = err {
                        return Err(ConsumerError::StartStateError(err));
                    }
                }

                // update job
                while consumer.job.is_continue() {
                    if let Err(err) = consumer.job.update().await {
                        let err = if let Err(handler_err) = consumer
                            .job
                            .error_handler(ConsumerError::UpdateStateError(err))
                            .await
                        {
                            warn!("{} error: message:{}", consumer.job.job_name(), handler_err);
                            Some(handler_err)
                        } else {
                            None
                        };

                        // if error handler failed return break the thread
                        if let Some(err) = err {
                            return Err(ConsumerError::UpdateStateError(err));
                        }
                    }
                }

                // end job
                if let Err(err) = consumer.job.end().await {
                    let err = if let Err(handler_err) = consumer
                        .job
                        .error_handler(ConsumerError::EndStateError(err))
                        .await
                    {
                        warn!("{} error: message:{}", consumer.job.job_name(), handler_err);
                        Some(handler_err)
                    } else {
                        None
                    };

                    // if error handler failed return break the thread
                    if let Some(err) = err {
                        return Err(ConsumerError::EndStateError(err));
                    }
                }

                Ok(())
            });

            match task.await {
                Ok(Ok(_)) => {
                    info!("{} 任務完成", self_move.lock().await.job.job_name());
                    return;
                }
                Ok(Err(err)) => {
                    warn!(
                        "{} execute error: {}",
                        self_move.lock().await.job.job_name(),
                        err
                    );
                }
                Err(err) => {
                    error!(
                        "{} task error:{}",
                        self_move.lock().await.job.job_name(),
                        err
                    );
                }
            }

            retries -= 1;
            if retries > 0 {
                warn!("重試consumer... 剩餘重試次數: {}", retries);
            } else {
                warn!("重試次數已用完... 結束consumer...");
                return;
            }
        }
    }
}
