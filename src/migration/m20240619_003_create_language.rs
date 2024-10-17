use crate::entity;
use crate::enums;
use chrono::Utc;
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
            .drop_table(Table::drop().table(Language::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Language {
    Table,
    Id,
    Name,
    CreateAt,
    UpdateAt,
}

impl Migration {
    async fn create_table(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Language::Table)
                    .if_not_exists()
                    .col(integer(Language::Id).primary_key())
                    .col(string(Language::Name).not_null())
                    .col(timestamp(Language::CreateAt).default(Expr::current_timestamp()))
                    .col(timestamp(Language::UpdateAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        let sql = r#"
            Comment on table language is '語言表';
            Comment on column language.id is 'ID';
            Comment on column language.name is '語言名稱';
            Comment on column language.create_at is '建立時間';
            Comment on column language.update_at is '更新時間';
        "#;

        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }

    async fn create_initial_data(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        let db = manager.get_connection();

        for language in enums::Language::iter() {
            // check if language is not exist
            if entity::language::Entity::find_by_id(language.to_id())
                .one(db)
                .await?
                .is_none()
            {
                // create entity
                let entity = entity::language::ActiveModel {
                    id: Set(language.to_id()),
                    name: Set(language.to_string()),
                    create_at: Set(Utc::now().naive_utc()),
                    update_at: Set(Utc::now().naive_utc()),
                };

                // insert entity
                entity::language::Entity::insert(entity).exec(db).await?;
            }
        }

        Ok(())
    }
}
