use crate::{enums, mq_manager};
use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "backstage_send_task")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub client_id: i64,                   // 發送目標的client id(前台client id)
    pub client_event_id: i64,             // client的事件id
    pub sender_id: i64,                   // 通知發送者id
    pub sender_account: String,           // 通知發送者帳號
    pub sender_ip: Option<String>,        // 通知發送者ip
    pub receiver_count: i32,              // 通知對象數量
    pub receiver_account: Vec<String>,    // 通知對象帳號
    pub receiver_id: Vec<i64>,            // 通知對象id
    pub task_name: String,                // 任務名稱
    pub notify_level: enums::NotifyLevel, // 通知等級 1.一般 2.系統 3.重要
    pub task_status: enums::TaskStatus,   // 任務狀態 1.待處理 2.成功 3.失敗
    pub error_message: Option<String>,    // 錯誤訊息
    pub create_at: NaiveDateTime,         // 建立時間
    pub update_at: NaiveDateTime,         // 更新時間
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn to_publish_model(
        self,
        frontend_client_id: i64,
        templates: Vec<mq_manager::TemplateModel>,
    ) -> crate::mq_manager::BatchNotifyModel {
        crate::mq_manager::BatchNotifyModel {
            task_id: self.id,
            client_id: self.client_id,
            sender_id: self.sender_id,
            sender_account: self.sender_account,
            sender_ip: self.sender_ip,
            notify_level: self.notify_level as i32,
            receiver_ids: self.receiver_id,
            templates: templates,
            client_event_id: self.client_event_id,
            frontend_client_id,
        }
    }
}

impl Model {
    pub fn to_proto<T: ProtoTrait>(self) -> T {
        T::to_proto(self)
    }
}

pub trait ProtoTrait {
    fn to_proto(model: Model) -> Self;
}

impl ProtoTrait for protos::backstage_notify::NotifyTask {
    fn to_proto(model: Model) -> Self {
        protos::backstage_notify::NotifyTask {
            task_id: model.id,
            task_name: model.task_name,
            sender_account: model.sender_account,
            sender_ip: model.sender_ip.unwrap_or_default(),
            receiver_count: model.receiver_count,
            receiver_account: model.receiver_account,
            notify_level: model.notify_level as i32,
            task_status: model.task_status as i32,
            error_message: model.error_message,
            create_at: model.create_at.and_utc().timestamp_millis(),
            update_at: model.update_at.and_utc().timestamp_millis(),
        }
    }
}
