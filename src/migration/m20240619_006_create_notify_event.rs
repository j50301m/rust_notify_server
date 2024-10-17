use crate::entity::{self, notify_event};
use crate::enums;
use sea_orm::{EntityTrait, Iterable, Set};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        self.create_table(manager).await?;

        self.create_initial_data(manager).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NotifyEvent::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum NotifyEvent {
    Table,
    Id,
    Platform,
    Name,
    Memo,
}

impl Migration {
    async fn create_table(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NotifyEvent::Table)
                    .if_not_exists()
                    .col(integer(NotifyEvent::Id).primary_key())
                    .col(string(NotifyEvent::Name).not_null())
                    .col(integer(NotifyEvent::Platform).not_null())
                    .col(string(NotifyEvent::Memo))
                    .to_owned(),
            )
            .await?;

        let sql = r#"
            Comment on table notify_event is '事件類型表';
            Comment on column notify_event.id is 'ID';
            Comment on column notify_event.name is '事件名稱';
            Comment on column notify_event.platform is '通知平台';
            Comment on column notify_event.memo is '備註';
        "#;

        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }

    async fn create_initial_data(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        let db = manager.get_connection();

        for event_type in enums::NotifyEvent::iter() {
            // Check if the record already exists
            if notify_event::Entity::find_by_id(event_type.to_id())
                .one(db)
                .await?
                .is_none()
            {
                // create entity
                let entity = entity::notify_event::ActiveModel {
                    id: Set(event_type.to_id()),
                    name: Set(event_type.to_string()),
                    platform: Set(event_type.get_platform()),
                    memo: Set(event_type.get_comment()),
                };

                // insert entity
                entity::notify_event::Entity::insert(entity)
                    .exec(db)
                    .await?;
            }
        }

        Ok(())
    }
}
