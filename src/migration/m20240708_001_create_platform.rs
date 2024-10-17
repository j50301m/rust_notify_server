use crate::entity::platform;
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
            .drop_table(Table::drop().table(Platform::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Platform {
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
                    .table(Platform::Table)
                    .if_not_exists()
                    .col(integer(Platform::Id).primary_key())
                    .col(string(Platform::Name).not_null())
                    .col(string(Platform::Memo))
                    .to_owned(),
            )
            .await?;

        let sql = r#"
            Comment on table platform is '通知平台表';
            Comment on column platform.id is 'ID';
            Comment on column platform.name is '通知平台名稱';
            Comment on column platform.memo is '備註';
        "#;

        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }

    async fn create_initial_data(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        let db = manager.get_connection();

        for platform in enums::Platform::iter() {
            if platform::Entity::find_by_id(platform.to_id())
                .one(db)
                .await?
                .is_some()
            {
                continue;
            }

            // Insert the platform
            platform::Entity::insert(platform::ActiveModel {
                id: Set(platform.to_id()),
                name: Set(platform.to_string()),
                memo: Set(platform.get_comment()),
            })
            .exec(db)
            .await?;
        }

        Ok(())
    }
}
