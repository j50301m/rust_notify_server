use crate::entity::backstage_send_task;
use crate::enums;
use chrono::DateTime;
use kgs_err::models::status::Status as KgsStatus;
use kgs_tracing::warn;
use protos::backstage_notify;
use sea_orm::*;
use sea_query::Expr;

pub async fn update_task_status<C>(
    db: &C,
    task_id: i64,
    status: enums::TaskStatus,
    err_msg: Option<String>,
) -> Result<backstage_send_task::Model, KgsStatus>
where
    C: ConnectionTrait,
{
    let update_result = backstage_send_task::Entity::update_many()
        .col_expr(backstage_send_task::Column::TaskStatus, Expr::value(status))
        .col_expr(
            backstage_send_task::Column::ErrorMessage,
            Expr::value(err_msg),
        )
        .filter(backstage_send_task::Column::Id.eq(task_id))
        .exec_with_returning(db)
        .await
        .map_err(|err| {
            warn!("update task status error: {}", err);
            KgsStatus::InternalServerError
        })?;

    if update_result.is_empty() {
        warn!("update task status error: task_id not found");
        return Err(KgsStatus::DataNotFound);
    }

    Ok(update_result.into_iter().next().unwrap())
}

pub async fn get_notify_task_list<C>(
    db: &C,
    page_size: u64,
    now_page: u64,
    request: backstage_notify::GetNotifyTaskListRequest,
) -> Result<(Vec<backstage_send_task::Model>, u64, u64), KgsStatus>
where
    C: ConnectionTrait,
{
    let query = backstage_send_task::Entity::find()
        .filter(backstage_send_task::Column::ClientId.eq(request.client_id))
        .apply_if(request.title, |query, v| {
            if request.is_fuzzy {
                query.filter(backstage_send_task::Column::TaskName.like(format!("%{}%", v)))
            } else {
                query.filter(backstage_send_task::Column::TaskName.eq(v))
            }
        })
        .apply_if(request.sender_account, |query, v| {
            query.filter(backstage_send_task::Column::SenderAccount.eq(v))
        })
        .apply_if(request.start_at, |query, v| {
            query.filter(
                backstage_send_task::Column::CreateAt.gte(DateTime::from_timestamp_millis(v)),
            )
        })
        .apply_if(request.end_at, |query, v| {
            query.filter(
                backstage_send_task::Column::CreateAt.lte(DateTime::from_timestamp_millis(v)),
            )
        })
        .order_by_desc(backstage_send_task::Column::CreateAt);

    // get total rows
    let total_rows = query.clone().count(db).await.map_err(|err| {
        warn!("get_notify_task_list count error: {}", err);
        KgsStatus::DataNotFound
    })?;

    // 創建分頁器
    let paginator = query.paginate(db, page_size);

    let records = paginator.fetch_page(now_page - 1).await.map_err(|err| {
        warn!("get_notify_task_list fetch_page error: {}", err);
        KgsStatus::DataNotFound
    })?;

    // 總頁數
    let total_pages = (total_rows as f64 / page_size as f64).ceil() as u64;

    Ok((records, total_rows, total_pages))
}
