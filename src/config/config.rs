use dotenv::dotenv;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

static CONFIG: Lazy<Arc<Config>> = Lazy::new(|| Config::init());

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub host: Host,
    pub telemetry: Telemetry,
    pub notification_db: NotificationDb,
    pub redis: Redis,
    pub rabbitmq: RabbitMQ,
    pub mailgun: MailGun,
    pub chuanxsms: ChuanxSMS,
    pub user_rpc: UserRpc,
    pub oauth_rpc: OauthRpc,
    pub kubernetes: Kubernetes,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Host {
    pub service_host: String,
    pub service_port: u32,
    pub service_name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Telemetry {
    pub loki_url: String,
    pub otlp_url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Redis {
    pub redis_host: String,
    pub redis_port: u32,
    pub redis_auth: String,
    pub redis_database: u32,
    pub redis_max_size: u32,
    pub redis_min_idle: u32,
    pub redis_connection_timeout: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NotificationDb {
    pub notify_db_host: String,
    pub notify_db_port: String,
    pub notify_db_user: String,
    pub notify_db_password: String,
    pub notify_db_name: String,
    pub notify_db_max_connection: u32,
    pub notify_db_min_connection: u32,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RabbitMQ {
    pub rabbitmq_host: String,
    pub rabbitmq_port: usize,
    pub rabbitmq_user: String,
    pub rabbitmq_password: String,
    pub rabbitmq_max_connection: usize,
    pub rabbitmq_min_connection: usize,
    pub rabbitmq_connection_timeout: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MailGun {
    pub mailgun_api_key: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChuanxSMS {
    pub appkey: String,
    pub appsecret: String,
    pub appcode: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UserRpc {
    pub user_server_host: String,
    pub user_server_port: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OauthRpc {
    pub oauth_server_host: String,
    pub oauth_server_port: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Kubernetes {
    pub pod_ip: String,
    pub pod_namespace: String,
    pub deployment_name: String,
}

impl Config {
    fn init() -> Arc<Config> {
        dotenv().ok();

        // if in k8s, use hostname as service name
        let mut host = envy::from_env::<Host>().expect("load host config error");
        if let Ok(name) = std::env::var("HOSTNAME") {
            host.service_name = name;
        }

        let telemetry = envy::from_env::<Telemetry>().expect("load telemetry config error");
        let notification_db =
            envy::from_env::<NotificationDb>().expect("load notification db config error");
        let redis = envy::from_env::<Redis>().expect("load redis config error");
        let rabbitmq = envy::from_env::<RabbitMQ>().expect("load rabbitmq config error");
        let mailgun = envy::from_env::<MailGun>().expect("load mailgun config error");
        let chuanxsms = envy::from_env::<ChuanxSMS>().expect("load chuanxsms config error");
        let user_rpc = envy::from_env::<UserRpc>().expect("load user config error");
        let oauth_rpc = envy::from_env::<OauthRpc>().expect("load oauth config error");
        let kubernetes = Kubernetes {
            pod_ip: std::env::var("POD_IP").unwrap_or(host.service_host.clone()),
            pod_namespace: std::env::var("POD_NAMESPACE").unwrap_or_default(),
            deployment_name: std::env::var("DEPLOYMENT_NAME").unwrap_or_default(),
        };
        let config = Config {
            host,
            telemetry,
            notification_db,
            redis,
            rabbitmq,
            mailgun,
            chuanxsms,
            user_rpc,
            oauth_rpc,
            kubernetes,
        };
        Arc::new(config)
    }
}

pub fn get_host() -> &'static Host {
    &CONFIG.host
}

pub fn get_telemetry() -> &'static Telemetry {
    &CONFIG.telemetry
}

pub fn get_notification_db() -> &'static NotificationDb {
    &CONFIG.notification_db
}

pub fn get_redis() -> &'static Redis {
    &CONFIG.redis
}

pub fn get_rabbit() -> &'static RabbitMQ {
    &CONFIG.rabbitmq
}

pub fn get_mailgun() -> &'static MailGun {
    &CONFIG.mailgun
}

pub fn get_chuanxsms() -> &'static ChuanxSMS {
    &CONFIG.chuanxsms
}

pub fn get_user_rpc() -> &'static UserRpc {
    &CONFIG.user_rpc
}

pub fn get_oauth_rpc() -> &'static OauthRpc {
    &CONFIG.oauth_rpc
}

pub fn get_kubernetes() -> &'static Kubernetes {
    &CONFIG.kubernetes
}
