use super::{batch_notify_job::BatchNotifyJob, consumer};
use crate::consumers::single_notify_job::SingleNotifyJob;
use kgs_tracing::tracing;

#[tracing::instrument]
pub fn start() -> Vec<tokio::task::JoinHandle<()>> {
    // 單一通知消費者
    let single_notify_job_1 = SingleNotifyJob::new("single_notify_consumer_1");
    let single_notify_job_2 = SingleNotifyJob::new("single_notify_consumer_2");
    let single_notify_job_3 = SingleNotifyJob::new("single_notify_consumer_3");
    let single_notify_job_4 = SingleNotifyJob::new("single_notify_consumer_4");
    let single_notify_job_5 = SingleNotifyJob::new("single_notify_consumer_5");
    let single_notify_job_6 = SingleNotifyJob::new("single_notify_consumer_6");
    let single_notify_job_7 = SingleNotifyJob::new("single_notify_consumer_7");
    let single_notify_job_8 = SingleNotifyJob::new("single_notify_consumer_8");
    let single_notify_job_9 = SingleNotifyJob::new("single_notify_consumer_9");
    let single_notify_job_10 = SingleNotifyJob::new("single_notify_consumer_10");

    // 批次通知消費者
    let batch_notify_job_1 = BatchNotifyJob::new("batch_notify_consumer_1");
    let batch_notify_job_2 = BatchNotifyJob::new("batch_notify_consumer_2");

    let consumers = vec![
        consumer::Consumer::new(single_notify_job_1, 3),
        consumer::Consumer::new(single_notify_job_2, 3),
        consumer::Consumer::new(single_notify_job_3, 3),
        consumer::Consumer::new(single_notify_job_4, 3),
        consumer::Consumer::new(single_notify_job_5, 3),
        consumer::Consumer::new(single_notify_job_6, 3),
        consumer::Consumer::new(single_notify_job_7, 3),
        consumer::Consumer::new(single_notify_job_8, 3),
        consumer::Consumer::new(single_notify_job_9, 3),
        consumer::Consumer::new(single_notify_job_10, 3),
        consumer::Consumer::new(batch_notify_job_1, 3),
        consumer::Consumer::new(batch_notify_job_2, 3),
    ];

    consumers
        .into_iter()
        .map(|consumer| {
            tokio::spawn(async move {
                consumer.execute().await;
            })
        })
        .collect()
}
