use crate::entity::notify_status;
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
            .drop_table(Table::drop().table(NotifyStatus::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum NotifyStatus {
    Table,
    Id,
    Name,
    Memo,
}

impl Migration {
    async fn create_table(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NotifyStatus::Table)
                    .if_not_exists()
                    .col(integer(NotifyStatus::Id).primary_key())
                    .col(string(NotifyStatus::Name).not_null())
                    .col(string(NotifyStatus::Memo))
                    .to_owned(),
            )
            .await?;

        let sql = r#"
            Comment on table notify_status is '通知狀態表';
            Comment on column notify_status.id is 'ID';
            Comment on column notify_status.name is '通知狀態名稱';
            Comment on column notify_status.memo is '備註';
        "#;

        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }

    async fn create_initial_data(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        let db = manager.get_connection();

        for notify_status in enums::NotifyStatus::iter() {
            // check if the notify_status exists
            if notify_status::Entity::find_by_id(notify_status.to_id())
                .one(db)
                .await?
                .is_some()
            {
                continue;
            }

            // insert notify_status
            notify_status::Entity::insert(notify_status::ActiveModel {
                id: Set(notify_status.to_id()),
                name: Set(notify_status.to_string()),
                memo: Set(notify_status.get_comment()),
            })
            .exec(db)
            .await?;
        }

        Ok(())
    }
}
