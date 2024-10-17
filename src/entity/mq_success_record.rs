use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;

use crate::enums;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mq_success_record")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub notify_id: i64,
    pub client_id: i64,
    pub user_id: i64,
    pub sender_id: i64,
    pub notify_type: enums::NotifyType,
    pub title: String,
    pub content: String,
    pub create_at: NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
