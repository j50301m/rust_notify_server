use crate::{entity::mq_failed_record, enums};
use sea_orm::entity::prelude::*;

pub async fn create(
    db: &sea_orm::DatabaseTransaction,
    notify_id: Option<i64>,
    client_id: Option<i64>,
    user_id: Option<i64>,
    sender_id: Option<i64>,
    title: Option<String>,
    notify_type: Option<enums::NotifyType>,
    content: Option<String>,
    error_message: Option<String>,
) -> Result<mq_failed_record::Model, sea_orm::DbErr> {
    let active_model = mq_failed_record::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        notify_id: sea_orm::Set(notify_id),
        client_id: sea_orm::Set(client_id),
        user_id: sea_orm::Set(user_id),
        sender_id: sea_orm::Set(sender_id),
        title: sea_orm::Set(title),
        content: sea_orm::Set(content),
        notify_type: sea_orm::Set(notify_type),
        create_at: sea_orm::ActiveValue::NotSet,
        error_message: sea_orm::Set(error_message),
    };

    active_model.insert(db).await
}
