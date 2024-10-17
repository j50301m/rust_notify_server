use crate::entity::notify_type;
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
            .drop_table(Table::drop().table(NotifyType::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum NotifyType {
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
                    .table(NotifyType::Table)
                    .if_not_exists()
                    .col(integer(NotifyType::Id).primary_key())
                    .col(string(NotifyType::Name).not_null())
                    .col(string(NotifyType::Memo))
                    .to_owned(),
            )
            .await?;

        let sql = r#"
            Comment on table notify_type is '通知類型表';
            Comment on column notify_type.id is 'ID';
            Comment on column notify_type.name is '通知名稱';
            Comment on column notify_type.memo is '備註';
        "#;

        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }

    async fn create_initial_data(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        let db = manager.get_connection();

        for notify_type in enums::NotifyType::iter() {
            // check if the notify_type exists
            if notify_type::Entity::find_by_id(notify_type.to_id())
                .one(db)
                .await?
                .is_some()
            {
                continue;
            }

            // insert the notify_type
            notify_type::Entity::insert(notify_type::ActiveModel {
                id: Set(notify_type.to_id()),
                name: Set(notify_type.to_string()),
                memo: Set(notify_type.get_comment()),
            })
            .exec(db)
            .await?;
        }

        Ok(())
    }
}
