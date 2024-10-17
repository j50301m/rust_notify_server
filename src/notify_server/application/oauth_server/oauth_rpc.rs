use kgs_err::models::status::Status as KgsStatus;
use kgs_tracing::{tracing, warn};
use protos::client::{client_client::ClientClient, *};

use crate::config;

#[tracing::instrument]
async fn get_client() -> Result<ClientClient<tonic::transport::Channel>, KgsStatus> {
    let oauth_config = config::config::get_oauth_rpc();
    let oauth_rpc_url = format!(
        "{}:{}",
        oauth_config.oauth_server_host, oauth_config.oauth_server_port
    );

    ClientClient::connect(oauth_rpc_url).await.map_err(|err| {
        warn!("Failed to connect to client server: {:?}", err);
        KgsStatus::InternalServerError
    })
}

#[tracing::instrument]
pub async fn get_backstage_client(frontend_client_id: i64) -> Result<i64, KgsStatus> {
    let mut client = get_client().await?;

    let request =
        kgs_tracing::tonic::create_request_with_span(FrontendClient { frontend_client_id });

    let response = client.get_backstage_client(request).await.map_err(|err| {
        warn!("Failed to get admin client id: {:?}", err);
        kgs_err::models::status::tonic_to_kgs(err)
    })?;

    Ok(response.into_inner().backstage_client_id)
}

#[tracing::instrument]
pub async fn get_frontend_client(backstage_client_id: i64) -> Result<i64, KgsStatus> {
    let mut client = get_client().await?;

    let request = kgs_tracing::tonic::create_request_with_span(BackstageClient {
        backstage_client_id,
    });

    let response = client.get_frontend_client(request).await.map_err(|err| {
        warn!("Failed to get front client id: {:?}", err);
        kgs_err::models::status::tonic_to_kgs(err)
    })?;

    Ok(response.into_inner().frontend_client_id)
}
