use kgs_err::models::status::Status as KgsStatus;
use kgs_tracing::{tracing, warn};
use protos::player::{player_client::PlayerClient, *};

use crate::config;

#[tracing::instrument]
async fn get_client() -> Result<PlayerClient<tonic::transport::Channel>, KgsStatus> {
    let player_rpc_config = config::config::get_user_rpc();
    let player_rpc_url = format!(
        "{}:{}",
        player_rpc_config.user_server_host, player_rpc_config.user_server_port
    );

    PlayerClient::connect(player_rpc_url).await.map_err(|err| {
        warn!("Failed to connect to user server: {:?}", err);
        KgsStatus::InternalServerError
    })
}

#[tracing::instrument]
pub async fn get_user_profile(
    client_id: i64,
    user_id: i64,
) -> Result<GetUserProfileResponse, KgsStatus> {
    let mut client = get_client().await?;

    let request =
        kgs_tracing::tonic::create_request_with_span(GetUserProfileRequest { client_id, user_id });

    let response = client.get_user_profile(request).await.map_err(|err| {
        warn!("Failed to get user profile: {:?}", err);
        kgs_err::models::status::tonic_to_kgs(err)
    })?;

    Ok(response.into_inner())
}

#[tracing::instrument]
pub async fn get_accounts_by_client_id(client_id: i64) -> Result<UserAccountList, KgsStatus> {
    let mut client = get_client().await?;

    let request =
        kgs_tracing::tonic::create_request_with_span(GetAccountByClientRequest { client_id });

    let response = client
        .get_account_by_client_id(request)
        .await
        .map_err(|err| {
            warn!("Failed to get all user account: {:?}", err);
            kgs_err::models::status::tonic_to_kgs(err)
        })?;

    Ok(response.into_inner())
}

#[tracing::instrument]
pub async fn get_accounts_by_vip_level(
    client_id: i64,
    vip_level: Vec<i64>,
) -> Result<UserAccountList, KgsStatus> {
    let mut client = get_client().await?;

    let request = kgs_tracing::tonic::create_request_with_span(GetAccountByVipLevelRequest {
        client_id,
        vip_level,
    });

    let response = client
        .get_account_by_vip_level(request)
        .await
        .map_err(|err| {
            warn!("Failed to get all user account: {:?}", err);
            kgs_err::models::status::tonic_to_kgs(err)
        })?;

    Ok(response.into_inner())
}

#[tracing::instrument]
pub async fn get_accounts_by_user_ids(
    client_id: i64,
    user_ids: Vec<i64>,
) -> Result<UserAccountList, KgsStatus> {
    let mut client = get_client().await?;

    let request = kgs_tracing::tonic::create_request_with_span(GetAccountByUserIdsRequest {
        client_id,
        user_ids,
    });

    let response = client
        .get_account_by_user_ids(request)
        .await
        .map_err(|err| {
            warn!("Failed to get all user account: {:?}", err);
            kgs_err::models::status::tonic_to_kgs(err)
        })?;

    Ok(response.into_inner())
}

#[tracing::instrument]
pub async fn get_account_by_user_id(
    client_id: i64,
    user_id: i64,
) -> Result<UserAccount, KgsStatus> {
    let user_accounts = get_accounts_by_user_ids(client_id, vec![user_id]).await?;

    match user_accounts.user_accounts.get(0) {
        Some(account) => Ok(account.to_owned()),
        None => Err(KgsStatus::UserNotFound),
    }
}

#[tracing::instrument]
pub async fn get_email_and_phone_by_user_ids(
    client_id: i64,
    user_ids: &Vec<i64>,
) -> Result<EmailAndPhoneList, KgsStatus> {
    let mut client = get_client().await?;

    let request = kgs_tracing::tonic::create_request_with_span(GetEmailAndPhoneByUserIdsRequest {
        client_id,
        user_ids: user_ids.clone(),
    });

    let response: tonic::Response<EmailAndPhoneList> = client
        .get_email_and_phone_by_user_ids(request)
        .await
        .map_err(|err| {
            warn!("Failed to get email and phone by user ids: {:?}", err);
            kgs_err::models::status::tonic_to_kgs(err)
        })?;

    Ok(response.into_inner())
}
