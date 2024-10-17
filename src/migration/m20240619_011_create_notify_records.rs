use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        self.create_table(manager).await?;

        self.add_update_trigger(manager).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NotifyRecord::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum NotifyRecord {
    Table,
    Id,
    ClientId,
    UserId,
    UserAccount,
    ClientNotifyEventId,
    SenderId,
    SenderAccount,
    SenderIp,
    NotifyType,
    NotifyLevel,
    NotifyStatus,
    Title,
    Content,
    CreateAt,
    UpdateAt,
}

impl Migration {
    async fn create_table(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NotifyRecord::Table)
                    .if_not_exists()
                    .col(big_integer(NotifyRecord::Id).not_null().primary_key())
                    .col(big_integer(NotifyRecord::ClientId).not_null())
                    .col(big_integer(NotifyRecord::UserId).not_null())
                    .col(string(NotifyRecord::UserAccount).not_null())
                    .col(big_integer(NotifyRecord::ClientNotifyEventId).not_null())
                    .col(big_integer(NotifyRecord::SenderId).not_null())
                    .col(string(NotifyRecord::SenderAccount).not_null())
                    .col(string_null(NotifyRecord::SenderIp))
                    .col(integer(NotifyRecord::NotifyType).not_null())
                    .col(integer(NotifyRecord::NotifyLevel).not_null())
                    .col(integer(NotifyRecord::NotifyStatus).not_null())
                    .col(string(NotifyRecord::Title).not_null())
                    .col(text(NotifyRecord::Content).not_null())
                    .col(timestamp(NotifyRecord::CreateAt).default(Expr::current_timestamp()))
                    .col(timestamp(NotifyRecord::UpdateAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        let sql = r#"
            Comment on table notify_record is '通知記錄表';
            Comment on column notify_record.id is 'ID';
            Comment on column notify_record.client_id is '客戶端ID';
            Comment on column notify_record.user_id is '用戶ID';
            Comment on column notify_record.user_account is '用戶帳號';
            Comment on column notify_record.client_notify_event_id is '客戶端通知事件ID';
            Comment on column notify_record.sender_id is '發送者ID';
            Comment on column notify_record.sender_account is '發送者帳號';
            Comment on column notify_record.sender_ip is '發送者IP';
            Comment on column notify_record.notify_type is '通知類型';
            Comment on column notify_record.notify_level is '通知等級';
            Comment on column notify_record.notify_status is '通知狀態';
            Comment on column notify_record.title is '標題';
            Comment on column notify_record.content is '內容';
            Comment on column notify_record.create_at is '建立時間';
            Comment on column notify_record.update_at is '更新時間';
            "#;

        manager.get_connection().execute_unprepared(sql).await?;
        Ok(())
    }

    async fn add_update_trigger(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        let sql = r#"
            CREATE TRIGGER trigger_update_timestamp
            BEFORE UPDATE ON notify_record
            FOR EACH ROW
            EXECUTE FUNCTION update_timestamp();
        "#;

        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }
}
