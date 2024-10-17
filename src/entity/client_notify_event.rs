use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;

use crate::enums::{self, NotifyType};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "client_notify_event")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub client_id: i64,
    pub platform: enums::Platform, // 此事件通知通知平台 1.前台 2.後台 3.總管理後台
    pub is_system_event: bool,     // 判別是不是系統預設事件 如果是則不可刪除與修改 id與name
    pub name: String,
    pub memo: String,
    pub notify_types: Option<Vec<NotifyType>>, // 此事件支援的通知類型
    pub editor_account: String,                // 編輯者帳號
    pub create_at: NaiveDateTime,
    pub update_at: NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    ClientNotifyTemplate,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::ClientNotifyTemplate => Entity::belongs_to(super::client_notify_template::Entity)
                .from(Column::Id)
                .to((
                    super::client_notify_template::Column::ClientNotifyEvent,
                    super::client_notify_template::Column::ClientId,
                ))
                .into(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn to_proto<T: ProtoTrait>(self) -> T {
        T::to_proto(self)
    }
}

pub trait ProtoTrait {
    fn to_proto(model: Model) -> Self;
}

impl ProtoTrait for protos::backstage_notify::ClientEventSummary {
    fn to_proto(model: Model) -> Self {
        protos::backstage_notify::ClientEventSummary {
            client_event_id: model.id,
            client_id: model.client_id,
            event_name: model.name,
        }
    }
}

impl ProtoTrait for protos::backstage_notify::ClientEvent {
    fn to_proto(model: Model) -> Self {
        protos::backstage_notify::ClientEvent {
            client_event_id: model.id,
            client_id: model.client_id,
            event_name: model.name,
            event_memo: model.memo,
            notify_types: model
                .notify_types
                .unwrap_or_default()
                .iter()
                .map(|x| x.to_id())
                .collect(),
            editor_account: model.editor_account,
            platform: model.platform.to_id(),
            is_system: model.is_system_event,
            create_at: model.create_at.and_utc().timestamp_millis(),
            update_at: model.update_at.and_utc().timestamp_millis(),
        }
    }
}
