use crate::entity::notify_level;
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
            .drop_table(Table::drop().table(NotifyLevel::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum NotifyLevel {
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
                    .table(NotifyLevel::Table)
                    .if_not_exists()
                    .col(integer(NotifyLevel::Id).primary_key())
                    .col(string(NotifyLevel::Name).not_null())
                    .col(string(NotifyLevel::Memo))
                    .to_owned(),
            )
            .await?;

        let sql = r#"
            Comment on table notify_level is '通知等級表';
            Comment on column notify_level.id is 'ID';
            Comment on column notify_level.name is '等級名稱';
            Comment on column notify_level.memo is '備註';
        "#;

        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }

    async fn create_initial_data(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        let db = manager.get_connection();

        for notify_level in enums::NotifyLevel::iter() {
            // check if the notify_level exists
            if notify_level::Entity::find_by_id(notify_level.to_id())
                .one(db)
                .await?
                .is_some()
            {
                continue;
            }

            // insert the notify_level
            notify_level::Entity::insert(notify_level::ActiveModel {
                id: Set(notify_level.to_id()),
                name: Set(notify_level.to_string()),
                memo: Set(notify_level.get_comment()),
            })
            .exec(db)
            .await?;
        }

        Ok(())
    }
}
