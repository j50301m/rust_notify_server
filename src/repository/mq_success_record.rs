use crate::entity::mq_success_record;
use crate::enums;
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, DatabaseTransaction, Set};

pub async fn create(
    db: &DatabaseTransaction,
    notify_id: i64,
    client_id: i64,
    user_id: i64,
    sender_id: i64,
    title: &str,
    content: &str,
    notify_type: &enums::NotifyType,
) -> Result<mq_success_record::Model, sea_orm::DbErr> {
    let active_model = mq_success_record::ActiveModel {
        id: NotSet,
        notify_id: Set(notify_id),
        client_id: Set(client_id),
        user_id: Set(user_id),
        sender_id: Set(sender_id),
        title: Set(title.to_string()),
        content: Set(content.to_string()),
        notify_type: Set(notify_type.clone()),
        create_at: NotSet,
    };

    active_model.insert(db).await
}
