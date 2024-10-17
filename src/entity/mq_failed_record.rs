use crate::enums;
use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mq_failed_record")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub notify_id: Option<i64>,
    pub client_id: Option<i64>,
    pub user_id: Option<i64>,
    pub sender_id: Option<i64>,
    pub notify_type: Option<enums::NotifyType>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub error_message: Option<String>,
    pub create_at: NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
