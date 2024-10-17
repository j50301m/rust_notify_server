use crate::entity::backstage_send_task_detail;
use crate::enums;
use crate::helper;
use kgs_err::models::status::Status as KgsStatus;
use kgs_tracing::warn;
use sea_orm::*;

pub async fn insert<C>(
    txn: &C,
    task_id: i64,
    notify_type: enums::NotifyType,
    notify_level: enums::NotifyLevel,
    title: String,
    content: String,
) -> Result<backstage_send_task_detail::Model, KgsStatus>
where
    C: ConnectionTrait,
{
    let id = helper::generate_snowflake_id().await;
    let entity = backstage_send_task_detail::ActiveModel {
        id: Set(id),
        backstage_send_task_id: Set(task_id),
        notify_type: Set(notify_type),
        notify_level: Set(notify_level),
        title: Set(title),
        content: Set(content),
    }
    .insert(txn)
    .await
    .map_err(|e| {
        warn!("insert notify_task_detail failed: {:?}", e);
        KgsStatus::InternalServerError
    })?;
    Ok(entity)
}

pub async fn find_list_by_task_id<C>(
    db: &C,
    task_id: i64,
) -> Result<Vec<backstage_send_task_detail::Model>, KgsStatus>
where
    C: ConnectionTrait,
{
    let result = backstage_send_task_detail::Entity::find()
        .filter(backstage_send_task_detail::Column::BackstageSendTaskId.eq(task_id))
        .all(db)
        .await
        .map_err(|_| KgsStatus::DataNotFound)?;
    Ok(result)
}
