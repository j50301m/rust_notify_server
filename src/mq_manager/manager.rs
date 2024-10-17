use deadpool::managed::QueueMode;
use deadpool_lapin::Connection;
use kgs_tracing::tracing;
use lapin::{
    message::Delivery,
    options::{BasicAckOptions, BasicConsumeOptions, BasicQosOptions},
    types::FieldTable,
    Channel, ConnectionProperties, Consumer,
};
use once_cell::sync::OnceCell;
use std::sync::Arc;

use super::{model::SingleNotifyModel, BatchNotifyModel};

static MQ_CONNECTION_POOL: OnceCell<Arc<deadpool_lapin::Pool>> = OnceCell::new();

const EXCHANGE_NAME: &str = "notify_exchange";
const SINGLE_NOTIFY_QUEUE_NAME: &str = "single_notify_queue"; // 對單一玩家通知的queue
const SINGLE_NOTIFY_ROUTING_KEY: &str = "single_notify_routing_key"; // 對單一玩家通知的routing key
const BATCH_NOTIFY_ROUTING_KEY: &str = "batch_notify_routing_key"; // 對多玩家通知的routing key
const BATCH_NOTIFY_QUEUE_NAME: &str = "batch_notify_queue"; // 對多玩家通知的queue

pub struct Builder<'a> {
    pub host: &'a str,
    pub port: usize,
    pub user: &'a str,
    pub password: &'a str,
    pub max_connection: usize,
    pub min_connection: usize,
    pub connection_timeout: usize,
}

impl<'a> Builder<'a> {
    pub fn new() -> Self {
        Self {
            host: "localhost",
            port: 0,
            user: "user",
            password: "password",
            max_connection: 100,
            min_connection: 5,
            connection_timeout: 600,
        }
    }

    pub fn host(&mut self, host: &'a str) -> &mut Self {
        self.host = host;
        self
    }

    pub fn port(&mut self, port: usize) -> &mut Self {
        self.port = port;
        self
    }

    pub fn user(&mut self, user: &'a str) -> &mut Self {
        self.user = user;
        self
    }

    pub fn password(&mut self, password: &'a str) -> &mut Self {
        self.password = password;
        self
    }

    pub fn max_connection(&mut self, max_connection: usize) -> &mut Self {
        self.max_connection = max_connection;
        self
    }

    pub fn min_connection(&mut self, min_connection: usize) -> &mut Self {
        self.min_connection = min_connection;
        self
    }

    pub fn connection_timeout(&mut self, connection_timeout: usize) -> &mut Self {
        self.connection_timeout = connection_timeout;
        self
    }

    pub async fn build(&self) -> &Arc<deadpool_lapin::Pool> {
        let url = format!(
            "amqp://{}:{}@{}:{}/%2f",
            self.user, self.password, self.host, self.port
        );

        let timeout = deadpool::managed::Timeouts {
            wait: Some(std::time::Duration::new(self.connection_timeout as u64, 0)),
            create: Some(std::time::Duration::new(self.connection_timeout as u64, 0)),
            recycle: Some(std::time::Duration::new(self.connection_timeout as u64, 0)),
        };

        let pool_config = deadpool::managed::PoolConfig {
            max_size: self.max_connection,
            timeouts: timeout,
            queue_mode: QueueMode::Fifo,
        };

        let cfg = deadpool_lapin::Config {
            url: Some(url),
            pool: Some(pool_config),
            connection_properties: ConnectionProperties::default(),
        };

        let pool = cfg
            .create_pool(Some(deadpool::Runtime::Tokio1))
            .expect("Failed to create rabbit connection pool");
        MQ_CONNECTION_POOL
            .set(Arc::new(pool))
            .expect("Failed to set rabbit connection pool");

        // init rabbit mq
        let _ = init_rabbit_mq().await.expect("Failed to init rabbit mq");

        // return rabbit connection pool
        MQ_CONNECTION_POOL
            .get()
            .expect("Failed to get rabbit connection pool")
    }
}

#[tracing::instrument]
async fn get_connection() -> Result<Connection, deadpool::managed::PoolError<lapin::Error>> {
    let pool = MQ_CONNECTION_POOL
        .get()
        .expect("please init rabbit_mq connection pool first");
    pool.get().await
}

async fn open_channel() -> Result<Channel, deadpool::managed::PoolError<lapin::Error>> {
    let conn = get_connection().await?;
    let channel = conn.create_channel().await?;
    Ok(channel)
}

#[tracing::instrument]
async fn init_rabbit_mq() -> Result<(), Box<dyn std::error::Error>> {
    let channel = open_channel().await?;

    // declare exchange
    let exchange_opt = lapin::options::ExchangeDeclareOptions {
        durable: true,
        ..Default::default()
    };
    channel
        .exchange_declare(
            EXCHANGE_NAME,
            lapin::ExchangeKind::Direct,
            exchange_opt,
            FieldTable::default(),
        )
        .await?;

    // declare queue
    let queue_opt = lapin::options::QueueDeclareOptions {
        durable: true,
        ..Default::default()
    };
    channel
        .queue_declare(SINGLE_NOTIFY_QUEUE_NAME, queue_opt, FieldTable::default())
        .await?;
    channel
        .queue_declare(BATCH_NOTIFY_QUEUE_NAME, queue_opt, FieldTable::default())
        .await?;

    // bind queue to exchange
    let bind_opt = lapin::options::QueueBindOptions::default();
    channel
        .queue_bind(
            SINGLE_NOTIFY_QUEUE_NAME,
            EXCHANGE_NAME,
            SINGLE_NOTIFY_ROUTING_KEY,
            bind_opt,
            FieldTable::default(),
        )
        .await?;
    channel
        .queue_bind(
            BATCH_NOTIFY_QUEUE_NAME,
            EXCHANGE_NAME,
            BATCH_NOTIFY_ROUTING_KEY,
            bind_opt,
            FieldTable::default(),
        )
        .await?;

    Ok(())
}

#[tracing::instrument]
/// Publish a message to rabbit mq
pub async fn publish_single_notify(
    message: &SingleNotifyModel,
) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    let channel = open_channel().await?;

    let payload = serde_json::to_string(message)?;
    let publish_opt = lapin::options::BasicPublishOptions::default();
    channel
        .basic_publish(
            EXCHANGE_NAME,
            SINGLE_NOTIFY_ROUTING_KEY,
            publish_opt,
            payload.as_bytes(),
            lapin::BasicProperties::default(),
        )
        .await?;

    Ok(())
}

#[tracing::instrument]
/// Publish a batch notify message to rabbit mq
pub async fn publish_batch_notify(
    message: &BatchNotifyModel,
) -> Result<(), Box<dyn std::error::Error>> {
    let channel = open_channel().await?;

    let payload = serde_json::to_string(message)?;
    let publish_opt = lapin::options::BasicPublishOptions::default();
    channel
        .basic_publish(
            EXCHANGE_NAME,
            BATCH_NOTIFY_ROUTING_KEY,
            publish_opt,
            payload.as_bytes(),
            lapin::BasicProperties::default(),
        )
        .await?;

    Ok(())
}

#[tracing::instrument]
pub async fn consume_single_notify(
    consumer_tag: &str,
) -> Result<Consumer, deadpool::managed::PoolError<lapin::Error>> {
    let channel = open_channel().await?;

    // setting consumer
    let basic_qos = BasicQosOptions { global: true }; // 全部的consumer共享一個Qos
    channel.basic_qos(1, basic_qos).await?; // 一次只處理一個message
    let consume_opt = BasicConsumeOptions::default();

    // consume message
    channel
        .basic_consume(
            SINGLE_NOTIFY_QUEUE_NAME,
            consumer_tag,
            consume_opt,
            FieldTable::default(),
        )
        .await
        .map_err(|err| err.into())
}

#[tracing::instrument]
pub async fn consume_batch_notify(
    consumer_tag: &str,
) -> Result<Consumer, deadpool::managed::PoolError<lapin::Error>> {
    let channel = open_channel().await?;

    // setting consumer
    let basic_qos = BasicQosOptions { global: true }; // 全部的consumer共享一個Qos
    channel.basic_qos(1, basic_qos).await?; // 一次只處理一個message
    let consume_opt = BasicConsumeOptions::default();

    // consume message
    channel
        .basic_consume(
            BATCH_NOTIFY_QUEUE_NAME,
            consumer_tag,
            consume_opt,
            FieldTable::default(),
        )
        .await
        .map_err(|err| err.into())
}

#[tracing::instrument]
pub async fn rabbit_consumer_ack(delivery: &Delivery) -> Result<(), lapin::Error> {
    delivery.ack(BasicAckOptions::default()).await
}
