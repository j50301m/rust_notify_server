use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::enums;
/// 傳送給前台用戶的通知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleNotifyModel {
    pub client_id: i64,
    pub user_id: i64,
    pub notify_id: i64,
    pub sender_id: i64,
    pub sender_account: String,
    pub sender_ip: Option<String>,
    pub notify_type: enums::NotifyType,
    pub notify_level: enums::NotifyLevel,
    pub title: String,
    pub content: String,
    pub receive_address: String,
    pub key_map: HashMap<String, String>,
    pub client_event_id: i64,
}

impl Default for SingleNotifyModel {
    fn default() -> Self {
        Self {
            notify_id: 0,
            client_id: 0,
            user_id: 0,
            sender_id: 0,
            sender_account: "".to_string(),
            sender_ip: None,
            notify_type: enums::NotifyType::InApp,
            notify_level: enums::NotifyLevel::Info,
            title: "".to_string(),
            content: "".to_string(),
            receive_address: "".to_string(),
            key_map: HashMap::new(),
            client_event_id: 0,
        }
    }
}

/// 後台使用者發送給前台用戶
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchNotifyModel {
    pub task_id: i64,
    pub frontend_client_id: i64,
    pub client_id: i64,
    pub client_event_id: i64,
    pub sender_id: i64,
    pub sender_account: String,
    pub sender_ip: Option<String>,
    pub notify_level: i32,
    pub receiver_ids: Vec<i64>,
    pub templates: Vec<TemplateModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateModel {
    pub notify_type: enums::NotifyType, // 通知管道 1.站內信 2.信箱 3.簡訊
    pub notify_level: enums::NotifyLevel, // 通知等級 1.一般通知 2.系統通知 3.重要通知
    pub title: String,                  // 尚未將{{代號}} 轉成對應參數
    pub content: String,                // 尚未將{{代號}} 轉成對應參數
}
