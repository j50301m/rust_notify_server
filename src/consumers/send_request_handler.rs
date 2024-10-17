use kgs_tracing::{tracing, warn};
use reqwest::Client;

use crate::config;
use crate::consumers::error::{ErrorKind, SendRequestError};

/// 轉換為日本電話號碼
#[tracing::instrument]
fn convert_to_jp_phone_number(address: &str) -> Result<String, SendRequestError> {
    if address.len() < 11 {
        return Err(SendRequestError::new(
            ErrorKind::InvalidPhoneNumber,
            "電話號碼格式錯誤".to_string(),
            None,
        ));
    }

    let country_code = &address[0..3];
    let rest = &address[3..];

    // 檢查rest的第一位是否為0 如果是則去掉這個0
    let phone: &str = if rest.starts_with('0') {
        &rest[1..]
    } else {
        rest
    };

    Ok(format!("{}{}", country_code, phone))
}

/// 發送emil實作方法
#[tracing::instrument]
pub async fn send_email(title: &str, content: &str, email: &str) -> Result<(), SendRequestError> {
    let client = Client::new();
    let response = client
        .post(&format!("https://api.mailgun.net/v3/{}/messages", "kgs.tw"))
        .basic_auth(
            "api",
            Some(config::config::get_mailgun().mailgun_api_key.clone()),
        )
        .form(&[
            ("from", "<mailgun@kgs.tw>"),
            ("to", email),
            ("subject", title),
            ("html", &content),
        ])
        .send()
        .await
        .map_err(|err| {
            warn!("Failed to send email: {:?}", err);
            SendRequestError::new(
                ErrorKind::ConnectionError,
                err.to_string(),
                Some(Box::new(err)),
            )
        })?;

    if !response.status().is_success() {
        let msg = format!("Failed to send email: {:#?}", response.text().await);
        warn!(msg);
        return Err(SendRequestError::new(ErrorKind::StatusError, msg, None));
    }

    Ok(())
}

/// 發送sms實作方法
#[tracing::instrument]
pub async fn send_sms(content: &str, address: &str) -> Result<(), SendRequestError> {
    // Send the GET request
    let client = Client::new();

    let phone = convert_to_jp_phone_number(address)?;

    let base_url = format!("http://47.242.85.7:9090/sms/batch/v2");
    let full_url = format!(
        "{}?appkey={}&appsecret={}&appcode={}&phone={}&msg={}&extend=",
        base_url,
        config::config::get_chuanxsms().appkey,
        config::config::get_chuanxsms().appsecret,
        config::config::get_chuanxsms().appcode,
        phone,
        &content
    );

    let response = client.get(&full_url).send().await.map_err(|err| {
        warn!("Failed to send sms: {:?}", err);
        SendRequestError::new(
            ErrorKind::ConnectionError,
            err.to_string(),
            Some(Box::new(err)),
        )
    })?;

    // Check if the request was successful
    if !response.status().is_success() {
        let msg = format!("Failed to send sms: {:#?}", response.text().await);
        warn!(msg);
        return Err(SendRequestError::new(ErrorKind::StatusError, msg, None));
    }

    Ok(())
}
