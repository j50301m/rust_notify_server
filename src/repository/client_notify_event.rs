use std::collections::HashSet;

use crate::entity::client_notify_event;
use crate::enums;
use crate::helper;

use kgs_err::models::status::Status as KgsStatus;
use kgs_tracing::warn;
use protos::backstage_notify;
use sea_orm::*;
use sea_query::Expr;

/// 查詢 client_notify_event entity 使用 client_id 和 notify_event_id
pub async fn get_client_notify_event_by_client_id_and_notify_event<C>(
    db: &C,
    client_id: i64,
    notify_event_id: i64,
) -> Result<client_notify_event::Model, KgsStatus>
where
    C: ConnectionTrait,
{
    client_notify_event::Entity::find()
        .filter(client_notify_event::Column::ClientId.eq(client_id))
        .filter(client_notify_event::Column::Id.eq(notify_event_id))
        .one(db)
        .await
        .map(|opt| {
            opt.ok_or(sea_orm::DbErr::RecordNotFound(format!(
                "client_notify_event not found for client_id: {}, id: {}",
                client_id, notify_event_id
            )))
        })
        .and_then(|res| res)
        .map_err(|_| KgsStatus::DataNotFound)
}

pub async fn get_list_by_client_id_and_is_system_and_platform<C>(
    db: &C,
    client_id: i64,
    platform: enums::Platform,
    is_system_event: bool,
) -> Result<Vec<client_notify_event::Model>, KgsStatus>
where
    C: ConnectionTrait,
{
    client_notify_event::Entity::find()
        .filter(client_notify_event::Column::ClientId.eq(client_id))
        .filter(client_notify_event::Column::Platform.eq(platform))
        .filter(client_notify_event::Column::IsSystemEvent.eq(is_system_event))
        .all(db)
        .await
        .map_err(|_| KgsStatus::DataNotFound)
}

/// 定義for前台的自定義事件
pub async fn create_custom_event<C>(
    db: &C,
    client_id: i64,
    editor_account: String,
    name: String,
    memo: String,
) -> Result<client_notify_event::Model, KgsStatus>
where
    C: ConnectionTrait,
{
    client_notify_event::ActiveModel {
        id: Set(helper::generate_snowflake_id().await),
        client_id: Set(client_id),
        platform: Set(enums::Platform::Frontend),
        name: Set(name),
        memo: Set(memo),
        notify_types: Set(Some(vec![
            enums::NotifyType::InApp,
            enums::NotifyType::Email,
        ])),
        is_system_event: Set(false),
        create_at: NotSet,
        update_at: NotSet,
        editor_account: Set(editor_account),
    }
    .insert(db)
    .await
    .map_err(|e| {
        warn!("Failed to create custom event: {:?}", e);
        KgsStatus::InternalServerError
    })
}

pub async fn get_list_by_get_client_event_request_proto<C>(
    db: &C,
    page_size: u64,
    now_page: u64,
    request: backstage_notify::GetClientEventRequest,
) -> Result<(Vec<client_notify_event::Model>, u64, u64), KgsStatus>
where
    C: ConnectionTrait,
{
    // 將notify_types轉換成字串
    let notify_type_str = request
        .notify_types
        .iter()
        .map(|notify_type| notify_type.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let query = client_notify_event::Entity::find()
        .filter(client_notify_event::Column::ClientId.eq(request.client_id))
        .apply_if(request.event_name, |query, v| {
            if request.is_fuzzy {
                query.filter(client_notify_event::Column::Name.like(format!("%{}%", v)))
            } else {
                query.filter(client_notify_event::Column::Name.eq(v))
            }
        })
        .apply_if(request.account, |query, v| {
            query.filter(client_notify_event::Column::EditorAccount.eq(v))
        })
        .apply_if(request.platform, |query, v| {
            query.filter(client_notify_event::Column::Platform.eq(v))
        })
        .apply_if(request.is_system, |query, v| {
            query.filter(client_notify_event::Column::IsSystemEvent.eq(v))
        })
        .apply_if(request.start_at, |query, v| {
            query.filter(client_notify_event::Column::UpdateAt.gte(v))
        })
        .apply_if(request.end_at, |query, v| {
            query.filter(client_notify_event::Column::UpdateAt.lte(v))
        })
        .filter(Expr::cust(format!(
            r#"client_notify_event.notify_types @> ARRAY[{}]"#,
            notify_type_str
        ))) // ＠> 表示包含於
        .order_by_desc(client_notify_event::Column::UpdateAt);

    // get total rows
    let total_rows = query.clone().count(db).await.map_err(|err| {
        warn!("get_client_notify_event_list count error: {}", err);
        KgsStatus::DataNotFound
    })?;

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

/// 刪除後台自定義事件 (不可以刪除系統事件)
pub async fn delete_custom_client_notify_event<C>(
    db: &C,
    client_id: i64,
    notify_event_id: i64,
) -> Result<(), KgsStatus>
where
    C: ConnectionTrait,
{
    // select the client_notify_event
    let entity = client_notify_event::Entity::find()
        .filter(client_notify_event::Column::ClientId.eq(client_id))
        .filter(client_notify_event::Column::Id.eq(notify_event_id))
        .filter(client_notify_event::Column::IsSystemEvent.ne(true))
        .filter(client_notify_event::Column::Platform.eq(enums::Platform::Frontend))
        .one(db)
        .await
        .map_err(|e| {
            warn!("Failed to find client_notify_event: {:?}", e);
            KgsStatus::DataNotFound
        })?
        .ok_or(KgsStatus::DataNotFound)?;

    // delete the client_notify_event
    entity.delete(db).await.map_err(|e| {
        warn!("Failed to delete client_notify_event: {:?}", e);
        KgsStatus::InternalServerError
    })?;

    Ok(())
}

pub async fn update_client_event<C>(
    db: &C,
    client_id: i64,
    notify_event_id: i64,
    name: Option<String>,
    notify_types: Vec<i32>,
    memo: Option<String>,
    editor_account: String,
) -> Result<(), KgsStatus>
where
    C: ConnectionTrait,
{
    // find entity
    let entity = client_notify_event::Entity::find()
        .filter(client_notify_event::Column::ClientId.eq(client_id))
        .filter(client_notify_event::Column::Id.eq(notify_event_id))
        .one(db)
        .await
        .map_err(|e| {
            warn!("Failed to find client_notify_event: {:?}", e);
            KgsStatus::DataNotFound
        })?
        .ok_or(KgsStatus::DataNotFound)?;

    // set name
    let mut active_model: client_notify_event::ActiveModel = entity.into();
    if let Some(name) = name {
        active_model.name = Set(name);
    }

    // set editor_account
    active_model.editor_account = Set(editor_account);

    // set memo
    if let Some(memo) = memo {
        active_model.memo = Set(memo);
    }

    // set notify_types
    if notify_types.is_empty() {
        active_model.notify_types = Set(None);
    } else {
        let notify_types: HashSet<i32> = notify_types.into_iter().collect(); // 去重複
        let notify_types = notify_types
            .iter()
            .map(|x| enums::NotifyType::try_from(*x))
            .collect::<Result<Vec<enums::NotifyType>, _>>()
            .map_err(|e| {
                warn!("Failed to convert notify_types: {:?}", e);
                KgsStatus::InvalidArgument
            })?;
        active_model.notify_types = Set(Some(notify_types));
    }

    active_model.update(db).await.map_err(|e| {
        warn!("Failed to update client_notify_event: {:?}", e);
        KgsStatus::InternalServerError
    })?;

    Ok(())
}
