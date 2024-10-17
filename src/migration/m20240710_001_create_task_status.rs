use crate::entity::task_status;
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
            .drop_table(Table::drop().table(TaskStatus::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TaskStatus {
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
                    .table(TaskStatus::Table)
                    .if_not_exists()
                    .col(integer(TaskStatus::Id).primary_key())
                    .col(string(TaskStatus::Name).not_null())
                    .col(string(TaskStatus::Memo))
                    .to_owned(),
            )
            .await?;

        let sql = r#"
            Comment on table task_status is '任務狀態表';
            Comment on column task_status.id is 'ID';
            Comment on column task_status.name is '狀態名稱';
            Comment on column task_status.memo is '備註';
        "#;

        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }

    async fn create_initial_data(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        let db = manager.get_connection();

        for task_status in enums::TaskStatus::iter() {
            // check if the notify_level exists
            if task_status::Entity::find_by_id(task_status.to_id())
                .one(db)
                .await?
                .is_some()
            {
                continue;
            }

            // insert the notify_level
            task_status::Entity::insert(task_status::ActiveModel {
                id: Set(task_status.to_id()),
                name: Set(task_status.to_string()),
                memo: Set(task_status.get_comment()),
            })
            .exec(db)
            .await?;
        }

        Ok(())
    }
}
