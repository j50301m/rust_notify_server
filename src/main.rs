use database_manager::redis;
use kgs_tracing::{info, tracing};
use sea_orm_migration::MigratorTrait;
mod config;
mod consumers;
mod entity;
mod enums;
mod helper;
mod migration;
mod mq_manager;
mod notify_server;
mod repository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_telemetry();
    init_db().await;
    init_redis();
    init_rabbit_mq().await;

    consumers::start();

    notify_grpc_server().await?;

    tokio::signal::ctrl_c().await.unwrap();
    Ok(())
}

#[tracing::instrument]
fn init_redis() {
    let config = config::config::get_redis();

    let options = redis::Options {
        host: config.redis_host.clone(),
        port: config.redis_port,
        database: config.redis_database,
        password: config.redis_auth.clone(),
        max_size: config.redis_max_size,
        min_idle: Some(config.redis_min_idle),
        connection_timeout: config.redis_connection_timeout,
    };

    let redis_manager = redis::RedisManager { option: options };
    redis_manager.new();
    info!("redis init success");
}

#[tracing::instrument]
async fn init_rabbit_mq() {
    let config = config::config::get_rabbit();
    let _ = mq_manager::Builder::new()
        .host(&config.rabbitmq_host)
        .port(config.rabbitmq_port)
        .user(&config.rabbitmq_user)
        .password(&config.rabbitmq_password)
        .max_connection(config.rabbitmq_max_connection)
        .min_connection(config.rabbitmq_min_connection)
        .connection_timeout(config.rabbitmq_connection_timeout)
        .build()
        .await;
}

#[tracing::instrument]
fn init_telemetry() {
    let host_config = config::config::get_host();
    let telemetry_config = config::config::get_telemetry();

    kgs_tracing::TelemetryBuilder::new(&host_config.service_name)
        .enable_log(&telemetry_config.loki_url)
        .enable_metrics(&telemetry_config.otlp_url)
        .enable_tracing(&telemetry_config.otlp_url)
        .build();

    // start metrics system CPU and RAM
    kgs_tracing::components::base_metrics::base_metrics(&host_config.service_name);
    info!("telemetry init success");
}

#[tracing::instrument]
async fn init_db() {
    let config = config::config::get_notification_db();
    let _db = database_manager::sea_orm::Builder::new()
        .db_user(&config.notify_db_user)
        .db_password(&config.notify_db_password)
        .db_host(&config.notify_db_host)
        .db_port(&config.notify_db_port)
        .db_name(&config.notify_db_name)
        .max_connections(config.notify_db_max_connection)
        .min_connections(config.notify_db_min_connection)
        .logging(true)
        .logging_level(database_manager::sea_orm::LogLevel::Info)
        .build()
        .await;

    // start migration
    // ref url: https://www.sea-ql.org/SeaORM/docs/next/migration/running-migration/
    migration::Migrator::up(&*_db, None)
        .await
        .expect("migration failed");
}

#[tracing::instrument]
async fn notify_grpc_server() -> Result<(), tonic::transport::Error> {
    use protos::backstage_notify::back_stage_notify_service_server::BackStageNotifyServiceServer;
    use protos::frontend_notify::frontend_notify_service_server::FrontendNotifyServiceServer;

    let config = config::config::get_host();
    let addr = format!("{}:{}", config.service_host, config.service_port)
        .parse()
        .unwrap();
    info!("start notify grpc server on {}", addr);

    tonic::transport::Server::builder()
        .layer(kgs_tracing::middlewares::tonic::root_span_builder())
        .layer(kgs_tracing::middlewares::tonic::TracingRecord::default())
        .add_service(FrontendNotifyServiceServer::from_arc(
            notify_server::FRONTEND_NOTIFY_SERVER.clone(),
        ))
        .add_service(BackStageNotifyServiceServer::from_arc(
            notify_server::BACKSTAGE_NOTIFY_SERVER.clone(),
        ))
        .serve(addr)
        .await
}
