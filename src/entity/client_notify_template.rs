use crate::enums;
use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "client_notify_template")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub client_id: i64,
    pub client_notify_event: i64,
    pub language_id: enums::Language,
    pub key_list: Option<Vec<String>>,
    pub notify_type: enums::NotifyType,
    pub title: String,
    pub content: String,
    pub is_system: bool, // 是否為系統訊息模板
    pub create_at: NaiveDateTime,
    pub update_at: NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Language,
    ClientNotifyEvent,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Language => Entity::belongs_to(super::language::Entity)
                .from(Column::LanguageId)
                .to(super::language::Column::Id)
                .into(),
            Self::ClientNotifyEvent => Entity::belongs_to(super::client_notify_event::Entity)
                .from((Column::ClientNotifyEvent, Column::ClientId))
                .to((
                    super::client_notify_event::Column::Id,
                    super::client_notify_event::Column::ClientId,
                ))
                .into(),
        }
    }
}

impl Related<super::language::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Language.def()
    }
}

impl Related<super::client_notify_event::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ClientNotifyEvent.def()
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

impl ProtoTrait for protos::backstage_notify::Template {
    fn to_proto(model: Model) -> Self {
        protos::backstage_notify::Template {
            title: model.title,
            content: model.content,
            notify_type: model.notify_type as i32,
        }
    }
}

impl ProtoTrait for protos::backstage_notify::TemplateWitKeyList {
    fn to_proto(model: Model) -> Self {
        let common_key = enums::CommonKey::get_all_keys(); // 所有模板都有的key
        protos::backstage_notify::TemplateWitKeyList {
            title: model.title,
            content: model.content,
            notify_type: model.notify_type as i32,
            keys: [common_key, model.key_list.unwrap_or_default()].concat(),
        }
    }
}
