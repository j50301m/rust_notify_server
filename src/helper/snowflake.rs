use kgs_tracing::tracing;
use once_cell::sync::Lazy;
use snowflake::SnowflakeIdGenerator;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::config;

static SNOWFLAKE_GENERATOR: Lazy<Arc<Mutex<SnowflakeIdGenerator>>> =
    Lazy::new(|| Arc::new(Mutex::new(get_snowflake_generator())));

#[tracing::instrument]
fn get_worker_id(pod_name: &str) -> u16 {
    let parts: Vec<&str> = pod_name.split('-').collect();
    let unique_part = parts.last().unwrap_or(&"1");
    let worker_id: u16 = unique_part.chars().fold(0, |acc, c| acc + c as u16);
    worker_id & 0x1F // 取低 5 位
}

#[tracing::instrument]
fn get_snowflake_generator() -> SnowflakeIdGenerator {
    let pod_name = &config::get_host().service_name;
    let worker_id = get_worker_id(&pod_name);

    SnowflakeIdGenerator::new(worker_id as i32, 1)
}

#[tracing::instrument]
pub async fn generate_id() -> i64 {
    let mut generator = SNOWFLAKE_GENERATOR.lock().await;
    generator.real_time_generate()
}
