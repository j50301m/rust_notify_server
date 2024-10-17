use crate::enums;
use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "notify_record")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub client_id: i64,
    pub user_id: i64,                       // 通知對象id
    pub user_account: String,               // 通知對象帳號
    pub client_notify_event_id: i64,        // client的通知事件id
    pub sender_id: i64,                     // 通知發送者id 0為系統
    pub sender_account: String,             // 通知發送者帳號 系統為System
    pub sender_ip: Option<String>,          // 通知發送者ip
    pub notify_type: enums::NotifyType,     // 通知管道 1.站內信 2.信箱 3.簡訊
    pub notify_level: enums::NotifyLevel,   // 通知等級
    pub notify_status: enums::NotifyStatus, // 通知狀態 1.未讀 2.已讀 3.已刪除
    pub title: String,                      // 通知標題
    pub content: String,                    // 通知內容
    pub create_at: NaiveDateTime,           // 通知建立時間
    pub update_at: NaiveDateTime,           // 更新時間
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn to_proto<T: ProtoTrait>(self) -> T {
        T::to_proto(self)
    }
}

pub trait ProtoTrait {
    fn to_proto(model: Model) -> Self;
}

impl ProtoTrait for protos::frontend_notify::Notify {
    fn to_proto(model: Model) -> Self {
        protos::frontend_notify::Notify {
            notify_id: model.id,
            notify_level: model.notify_level as i32,
            create_at: model.create_at.and_utc().timestamp_millis(),
            title: model.title,
            content: model.content,
            notify_status: model.notify_status as i32,
        }
    }
}

impl ProtoTrait for protos::backstage_notify::Notify {
    fn to_proto(model: Model) -> Self {
        protos::backstage_notify::Notify {
            notify_id: model.id,
            notify_level: model.notify_level as i32,
            create_at: model.create_at.and_utc().timestamp_millis(),
            title: model.title,
            content: model.content,
            notify_status: model.notify_status as i32,
        }
    }
}

impl ProtoTrait for protos::backstage_notify::UserNotifyRecord {
    fn to_proto(model: Model) -> Self {
        protos::backstage_notify::UserNotifyRecord {
            notify_id: model.id,
            title: model.title,
            receiver_account: model.user_account,
            notify_status: model.notify_status as i32,
            notify_type: model.notify_type as i32,
            notify_level: model.notify_level as i32,
            sender_ip: model.sender_ip,
            create_at: model.create_at.and_utc().timestamp_millis(),
            sender_account: model.sender_account,
            content: model.content,
        }
    }
}
