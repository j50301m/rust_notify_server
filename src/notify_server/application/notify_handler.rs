use crate::config;
use crate::enums;
use crate::repository;
use kgs_err::models::status::Status as KgsStatus;
use kgs_tracing::tracing;
use kgs_tracing::warn;
use redis::Commands;
use std::collections::HashMap;
use std::collections::HashSet;

const REDIS_KEY: &str = "notify_server";
const USER_EXPIRE_SECS: usize = 60 * 60 * 24 * 7;

#[tracing::instrument]
pub async fn is_platform_has_event<C>(
    db: &C,
    client_id: i64,
    notify_event: &enums::NotifyEvent,
    platform: enums::Platform,
) -> Result<bool, KgsStatus>
where
    C: sea_orm::ConnectionTrait + std::fmt::Debug,
{
    let notify_event_entity =
        repository::client_notify_event::get_client_notify_event_by_client_id_and_notify_event(
            db,
            client_id,
            notify_event.to_id(),
        )
        .await?;
    if notify_event_entity.platform == platform {
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tracing::instrument]
pub fn has_common_role(a: &Vec<i64>, b: &Vec<i64>) -> bool {
    let set_a: HashSet<_> = a.iter().cloned().collect();
    b.iter().any(|x| set_a.contains(x))
}

#[tracing::instrument]
pub fn replace_title_and_content(
    title: &str,
    content: &str,
    user_profile: &protos::player::GetUserProfileResponse,
    key_map: &HashMap<String, String>,
) -> (String, String) {
    // define function -replace key map
    fn replace_keys(text: &str, key_map: &HashMap<String, String>) -> String {
        key_map.iter().fold(text.to_string(), |acc, (key, value)| {
            acc.replace(key, value)
        })
    }

    // define function -replace template
    fn replace_template(text: &mut String, placeholder: &str, value: &str) {
        *text = text.replace(placeholder, value);
    }

    // replace by key_map
    let mut title = replace_keys(title, key_map);
    let mut content = replace_keys(content, key_map);

    // replace by user_profile
    let replacements = [
        (
            enums::CommonKey::UserAccount.get_key(),
            &user_profile.account,
        ),
        (
            enums::CommonKey::UserLastName.get_key(),
            &user_profile.last_name,
        ),
        (
            enums::CommonKey::UserFirstName.get_key(),
            &user_profile.first_name,
        ),
        (enums::CommonKey::UserCity.get_key(), &user_profile.city),
        (
            enums::CommonKey::UserCountry.get_key(),
            &user_profile.country,
        ),
    ];

    // replace template
    for (placeholder, value) in &replacements {
        replace_template(&mut title, placeholder, value);
        replace_template(&mut content, placeholder, value);
    }

    (title, content)
}

#[tracing::instrument]
pub fn get_receive_address(notify_type: &enums::NotifyType, email: &str, phone: &str) -> String {
    match notify_type {
        enums::NotifyType::Email => email.to_string(),
        enums::NotifyType::SMS => phone.to_string(),
        _ => "".to_string(),
    }
}

#[tracing::instrument]
pub fn get_receive_address_opt(
    notify_type: &enums::NotifyType,
    email: &Option<String>,
    phone: &Option<String>,
) -> String {
    match notify_type {
        enums::NotifyType::Email => email.clone().unwrap_or_default(),
        enums::NotifyType::SMS => phone.clone().unwrap_or_default(),
        _ => "".to_string(),
    }
}

#[tracing::instrument]
pub fn save_user_located_server_to_redis(user_id: i64) -> Result<(), KgsStatus> {
    let key = format!("{}:{}", REDIS_KEY, user_id);
    let value = &config::config::get_kubernetes().pod_ip;
    let mut redis_conn = database_manager::redis::RedisManager::get_conn();
    redis_conn
        .set_ex(key, value, USER_EXPIRE_SECS as u64)
        .map_err(|e| {
            warn!("save user to redis failed: {}", e);
            KgsStatus::InternalServerError
        })?;

    Ok(())
}

#[tracing::instrument]
pub fn get_user_located_server_from_redis(user_id: i64) -> Result<Option<String>, KgsStatus> {
    let key = format!("{}:{}", REDIS_KEY, user_id);
    let mut redis_conn = database_manager::redis::RedisManager::get_conn();
    let value = redis_conn.get(key).map_err(|e| {
        warn!("get user location server from redis failed: {}", e);
        KgsStatus::InternalServerError
    })?;
    Ok(value)
}

#[tracing::instrument]
pub fn remove_user_located_server_from_redis(user_id: i64) -> Result<(), KgsStatus> {
    let key = format!("{}:{}", REDIS_KEY, user_id);
    let mut redis_conn = database_manager::redis::RedisManager::get_conn();
    redis_conn.del(key).map_err(|e| {
        warn!("remove user from redis failed: {}", e);
        KgsStatus::InternalServerError
    })?;

    Ok(())
}
