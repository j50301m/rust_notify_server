use crate::entity::notify_record;
use crate::enums;
use chrono::DateTime;
use kgs_err::models::status::Status as KgsStatus;
use kgs_tracing::warn;
use sea_orm::*;
use sea_query::Expr;

pub async fn find_one_by_id_and_client_id_user_id<C>(
    db: &C,
    id: i64,
    client_id: i64,
    user_id: i64,
) -> Result<notify_record::Model, KgsStatus>
where
    C: ConnectionTrait,
{
    let result = notify_record::Entity::find()
        .filter(notify_record::Column::Id.eq(id))
        .filter(notify_record::Column::ClientId.eq(client_id))
        .filter(notify_record::Column::UserId.eq(user_id))
        .one(db)
        .await
        .map_err(|_| KgsStatus::DataNotFound)?;

    result.ok_or(KgsStatus::DataNotFound)
}

/// 獲取所有 站內訊息
pub async fn find_all_app_record_by_user_id_and_status<C>(
    db: &C,
    client_id: i64,
    user_id: i64,
    notify_status: Option<enums::NotifyStatus>,
    notify_level: Option<enums::NotifyLevel>,
    now_page: u64,
    page_size: u64,
) -> Result<(Vec<notify_record::Model>, u64, u64), KgsStatus>
where
    C: ConnectionTrait,
{
    // 查詢條件
    let query = notify_record::Entity::find()
        .filter(notify_record::Column::ClientId.eq(client_id))
        .filter(notify_record::Column::UserId.eq(user_id))
        .filter(notify_record::Column::NotifyType.eq(enums::NotifyType::InApp))
        .filter(if let Some(notify_status) = notify_status {
            notify_record::Column::NotifyStatus.eq(notify_status)
        } else {
            notify_record::Column::NotifyStatus.ne(enums::NotifyStatus::Delete)
        })
        .apply_if(notify_level, |query, v| {
            query.filter(notify_record::Column::NotifyLevel.eq(v))
        })
        .order_by(notify_record::Column::CreateAt, Order::Desc);

    // 總筆數
    let total_rows = query
        .clone()
        .count(db)
        .await
        .map_err(|_| KgsStatus::DataNotFound)?;

    // 創建分頁器
    let paginator = query.paginate(db, page_size);

    // 取得當前頁數的資料
    let records = paginator
        .fetch_page(now_page - 1)
        .await
        .map_err(|_| KgsStatus::DataNotFound)?;

    // 總頁數
    let total_pages = (total_rows as f64 / page_size as f64).ceil() as u64;

    Ok((records, total_rows, total_pages))
}

pub async fn update_notify_records<C>(
    db: &C,
    client_id: i64,
    user_id: i64,
    notify_status: enums::NotifyStatus,
    notify_record_ids: Vec<i64>,
) -> Result<Vec<notify_record::Model>, KgsStatus>
where
    C: ConnectionTrait,
{
    notify_record::Entity::update_many()
        .col_expr(
            notify_record::Column::NotifyStatus,
            Expr::value(notify_status),
        )
        .filter(notify_record::Column::ClientId.eq(client_id))
        .filter(notify_record::Column::UserId.eq(user_id))
        .filter(notify_record::Column::Id.is_in(notify_record_ids))
        .exec_with_returning(db)
        .await
        .map_err(|_| KgsStatus::DataNotFound)
}

pub async fn get_unread_app_record_notify_count<C>(
    db: &C,
    client_id: i64,
    user_id: i64,
    notify_level: Option<enums::NotifyLevel>,
) -> Result<u64, KgsStatus>
where
    C: ConnectionTrait,
{
    // 查詢條件
    let query = notify_record::Entity::find()
        .filter(notify_record::Column::ClientId.eq(client_id))
        .filter(notify_record::Column::UserId.eq(user_id))
        .filter(notify_record::Column::NotifyType.eq(enums::NotifyType::InApp));
    let query = if let Some(notify_level) = notify_level {
        query.filter(notify_record::Column::NotifyLevel.eq(notify_level))
    } else {
        query
    };

    // 總筆數
    query.count(db).await.map_err(|_| KgsStatus::DataNotFound)
}

/// 獲取所有 站內未讀訊息
pub async fn get_all_unread_app_records_count<C>(
    db: &C,
    client_id: i64,
    user_id: i64,
) -> Result<u64, KgsStatus>
where
    C: ConnectionTrait,
{
    notify_record::Entity::find()
        .filter(notify_record::Column::ClientId.eq(client_id))
        .filter(notify_record::Column::UserId.eq(user_id))
        .filter(notify_record::Column::NotifyStatus.eq(enums::NotifyStatus::Unread))
        .filter(notify_record::Column::NotifyType.eq(enums::NotifyType::InApp))
        .count(db)
        .await
        .map_err(|_| KgsStatus::DataNotFound)
}

pub async fn update_all_with_notify_level<C>(
    db: &C,
    client_id: i64,
    user_id: i64,
    notify_level: Option<enums::NotifyLevel>,
    notify_status: enums::NotifyStatus,
) -> Result<Vec<notify_record::Model>, KgsStatus>
where
    C: ConnectionTrait,
{
    let result = notify_record::Entity::update_many()
        .col_expr(
            notify_record::Column::NotifyStatus,
            Expr::value(notify_status),
        )
        .filter(notify_record::Column::ClientId.eq(client_id))
        .filter(notify_record::Column::UserId.eq(user_id))
        .apply_if(notify_level, |query, v| {
            query.filter(notify_record::Column::NotifyLevel.eq(v))
        })
        .exec_with_returning(db)
        .await
        .map_err(|e| {
            warn!("update_all_with_notify_level error: {:?}", e);
            KgsStatus::InternalServerError
        })?;

    Ok(result)
}

pub async fn get_user_notify_records_for_backstage<C>(
    db: &C,
    page_size: u64,
    now_page: u64,
    request: protos::backstage_notify::GetUserNotifyRecordRequest,
) -> Result<(Vec<notify_record::Model>, u64, u64), KgsStatus>
where
    C: ConnectionTrait,
{
    let query = notify_record::Entity::find()
        .filter(notify_record::Column::ClientId.eq(request.client_id))
        .apply_if(request.title, |query, v| {
            if request.is_fuzzy {
                query.filter(notify_record::Column::Title.like(format!("%{}%", v)))
            } else {
                query.filter(notify_record::Column::Title.eq(v))
            }
        })
        .apply_if(request.receiver_account, |query, v| {
            query.filter(notify_record::Column::UserAccount.eq(v))
        })
        .apply_if(request.sender_account, |query, v| {
            query.filter(notify_record::Column::SenderAccount.eq(v))
        })
        .filter(notify_record::Column::NotifyStatus.is_in(request.notify_status))
        .filter(notify_record::Column::NotifyType.is_in(request.notify_type))
        .filter(notify_record::Column::NotifyLevel.is_in(request.notify_level))
        .apply_if(request.start_at, |query, v| {
            query.filter(notify_record::Column::CreateAt.gte(DateTime::from_timestamp_millis(v)))
        })
        .apply_if(request.end_at, |query, v| {
            query.filter(notify_record::Column::CreateAt.lte(DateTime::from_timestamp_millis(v)))
        })
        .order_by_desc(notify_record::Column::CreateAt);

    // 總筆數
    let total_rows = query
        .clone()
        .count(db)
        .await
        .map_err(|_| KgsStatus::DataNotFound)?;

    // 創建分頁器
    let paginator = query.paginate(db, page_size);

    // 取得當前頁數的資料
    let records = paginator
        .fetch_page(now_page - 1)
        .await
        .map_err(|_| KgsStatus::DataNotFound)?;

    // 總頁數
    let total_pages = (total_rows as f64 / page_size as f64).ceil() as u64;

    Ok((records, total_rows, total_pages))
}
