use std::collections::HashSet;
use std::vec;

use crate::entity::client_notify_template;
use crate::enums;
use kgs_err::models::status::Status as KgsStatus;
use kgs_tracing::warn;
use protos::backstage_notify;
use sea_orm::*;

/// 只拿出client event type 有開啟的模板
pub async fn find_list_by_client_id_and_notify_event_is_on<C>(
    db: &C,
    client_id: i64,
    notify_event_id: i64,
    language: &enums::Language,
) -> Result<Vec<client_notify_template::Model>, KgsStatus>
where
    C: ConnectionTrait,
{
    let sql = r#"
        SELECT cnt.*
        FROM client_notify_template cnt
        INNER JOIN client_notify_event cne
        ON cnt.client_notify_event = cne.id
        AND cne.client_id = cnt.client_id
        WHERE cnt.client_id = $1
        AND cnt.client_notify_event = $2
        AND cnt.notify_type = ANY(cne.notify_types)
        AND cnt.language_id = $3
    "#;

    client_notify_template::Entity::find()
        .from_raw_sql(Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            vec![
                client_id.into(),
                notify_event_id.into(),
                language.to_id().into(),
            ],
        ))
        .all(db)
        .await
        .map_err(|_| KgsStatus::DataNotFound)
}

pub async fn find_one_by_client_id_and_notify_event_and_notify_type_and_language<C>(
    db: &C,
    client_id: i64,
    notify_event_id: i64,
    notify_type: enums::NotifyType,
    language: &enums::Language,
) -> Result<client_notify_template::Model, KgsStatus>
where
    C: ConnectionTrait,
{
    let result = client_notify_template::Entity::find()
        .filter(client_notify_template::Column::ClientId.eq(client_id))
        .filter(client_notify_template::Column::ClientNotifyEvent.eq(notify_event_id))
        .filter(client_notify_template::Column::NotifyType.eq(notify_type))
        .filter(client_notify_template::Column::LanguageId.eq(language.to_id()))
        .one(db)
        .await
        .map_err(|_| KgsStatus::DataNotFound)?;

    result.ok_or(KgsStatus::DataNotFound)
}

/// 拿出所有 client event type 底下所有模板
pub async fn find_list_by_client_id_and_notify_event<C>(
    db: &C,
    client_id: i64,
    notify_event_id: i64,
    language: &enums::Language,
) -> Result<Vec<client_notify_template::Model>, KgsStatus>
where
    C: ConnectionTrait,
{
    let result = client_notify_template::Entity::find()
        .filter(client_notify_template::Column::ClientId.eq(client_id))
        .filter(client_notify_template::Column::ClientNotifyEvent.eq(notify_event_id))
        .filter(client_notify_template::Column::LanguageId.eq(language.to_id()))
        .all(db)
        .await
        .map_err(|_| KgsStatus::DataNotFound)?;

    Ok(result)
}

pub async fn create_custom_templates<C>(
    db: &C,
    client_id: i64,
    client_notify_event_id: i64,
    language: &enums::Language,
    templates: &Vec<backstage_notify::Template>,
) -> Result<(), KgsStatus>
where
    C: ConnectionTrait,
{
    // 去除重複的notify_type的template
    let mut set = HashSet::new();
    let mut unique_templates = vec![];
    for template in templates {
        if set.insert(template.notify_type) {
            unique_templates.push(template);
        }
    }

    let mut active_models = Vec::with_capacity(templates.len());
    for template in unique_templates {
        let model = client_notify_template::ActiveModel {
            id: NotSet,
            client_id: Set(client_id),
            client_notify_event: Set(client_notify_event_id),
            language_id: Set(language.clone()),
            title: Set(template.title.clone()),
            content: Set(template.content.clone()),
            notify_type: Set(enums::NotifyType::try_from(template.notify_type)?),
            is_system: Set(false),
            key_list: NotSet,
            create_at: NotSet,
            update_at: NotSet,
        };

        active_models.push(model);
    }

    client_notify_template::Entity::insert_many(active_models)
        .exec(db)
        .await
        .map_err(|e| {
            warn!("Failed to create custom templates: {:?}", e);
            KgsStatus::InternalServerError
        })?;

    Ok(())
}

pub async fn update_client_templates<C>(
    db: &C,
    client_id: i64,
    client_notify_event_id: i64,
    templates: &Vec<backstage_notify::Template>,
) -> Result<(), KgsStatus>
where
    C: ConnectionTrait,
{
    // select all templates
    let entities = client_notify_template::Entity::find()
        .filter(client_notify_template::Column::ClientId.eq(client_id))
        .filter(client_notify_template::Column::ClientNotifyEvent.eq(client_notify_event_id))
        .all(db)
        .await
        .map_err(|_| KgsStatus::DataNotFound)?;

    // for each entity, update the content
    for entity in entities {
        let template = templates
            .iter()
            .find(|t| t.notify_type == entity.notify_type.to_id());
        if let Some(template) = template {
            let mut active_model: client_notify_template::ActiveModel = entity.into();
            active_model.content = Set(template.content.to_owned());
            active_model.title = Set(template.title.to_owned());
            active_model.update(db).await.map_err(|e| {
                warn!("Failed to update custom templates: {:?}", e);
                KgsStatus::InternalServerError
            })?;
        }
    }

    Ok(())
}

/// 刪除後台自定義模板 (不可以刪除系統模板)
pub async fn delete_custom_templates<C>(
    db: &C,
    client_id: i64,
    client_notify_event_id: i64,
) -> Result<(), KgsStatus>
where
    C: ConnectionTrait,
{
    client_notify_template::Entity::delete_many()
        .filter(client_notify_template::Column::ClientId.eq(client_id))
        .filter(client_notify_template::Column::ClientNotifyEvent.eq(client_notify_event_id))
        .filter(client_notify_template::Column::IsSystem.eq(false))
        .exec(db)
        .await
        .map_err(|e| {
            warn!("Failed to delete custom templates: {:?}", e);
            KgsStatus::InternalServerError
        })?;

    Ok(())
}
