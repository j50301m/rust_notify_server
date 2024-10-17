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
            .drop_table(Table::drop().table(MqFailedRecord::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum MqFailedRecord {
    Table,
    Id,
    NotifyId,
    ClientId,
    UserId,
    SenderId,
    Title,
    Content,
    NotifyType,
    ErrorMessage,
    CreateAt,
}

impl Migration {
    async fn create_table(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MqFailedRecord::Table)
                    .if_not_exists()
                    .col(
                        big_integer(MqFailedRecord::Id)
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(big_integer_null(MqFailedRecord::NotifyId))
                    .col(big_integer_null(MqFailedRecord::ClientId))
                    .col(big_integer_null(MqFailedRecord::UserId))
                    .col(big_integer_null(MqFailedRecord::SenderId))
                    .col(text_null(MqFailedRecord::Title))
                    .col(text_null(MqFailedRecord::Content))
                    .col(text(MqFailedRecord::ErrorMessage).not_null())
                    .col(integer_null(MqFailedRecord::NotifyType))
                    .col(timestamp(MqFailedRecord::CreateAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        let sql = r#"
                    Comment on column mq_failed_record.notify_id is '通知id';
                    Comment on column mq_failed_record.client_id is '通知紀錄client id';
                    Comment on column mq_failed_record.user_id is '通知紀錄user id';
                    Comment on column mq_failed_record.sender_id is '發送者id';
                    Comment on column mq_failed_record.title is '通知標題';
                    Comment on column mq_failed_record.content is '通知內容';
                    Comment on column mq_failed_record.error_message is '錯誤訊息';
                    Comment on column mq_failed_record.notify_type is '通知類型';
                    Comment on column mq_failed_record.create_at is '創建時間';
            "#;
        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }
}
