use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        self.create_table(manager).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MqSuccessRecord::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum MqSuccessRecord {
    Table,
    Id,
    NotifyId,
    ClientId,
    SenderId,
    UserId,
    Title,
    Content,
    NotifyType,
    CreateAt,
}

impl Migration {
    async fn create_table(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MqSuccessRecord::Table)
                    .if_not_exists()
                    .col(
                        big_integer(MqSuccessRecord::Id)
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(big_integer(MqSuccessRecord::NotifyId))
                    .col(big_integer(MqSuccessRecord::ClientId))
                    .col(big_integer(MqSuccessRecord::UserId))
                    .col(big_integer(MqSuccessRecord::SenderId))
                    .col(text_null(MqSuccessRecord::Title))
                    .col(text_null(MqSuccessRecord::Content))
                    .col(integer(MqSuccessRecord::NotifyType))
                    .col(timestamp(MqSuccessRecord::CreateAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        let sql = r#"
            Comment on column mq_success_record.client_id is '通知紀錄client id';
            Comment on column mq_success_record.user_id is '通知紀錄user id';
            Comment on column mq_success_record.sender_id is '發送者id';
            Comment on column mq_success_record.title is '通知標題';
            Comment on column mq_success_record.content is '通知內容';
            Comment on column mq_success_record.create_at is '創建時間';
        "#;

        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }
}
