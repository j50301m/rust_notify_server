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
            .drop_table(Table::drop().table(BackstageSendTask::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum BackstageSendTask {
    Table,
    Id,
    ClientId,
    SenderId,
    SenderAccount,
    SenderIp,
    ReceiverCount,
    ReceiverAccount,
    ReceiverId,
    TaskName,
    NotifyLevel,
    TaskStatus,
    ClientEventId,
    ErrorMessage,
    CreateAt,
    UpdateAt,
}

impl Migration {
    async fn create_table(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BackstageSendTask::Table)
                    .if_not_exists()
                    .col(big_integer(BackstageSendTask::Id).not_null().primary_key())
                    .col(big_integer(BackstageSendTask::ClientId).not_null())
                    .col(big_integer(BackstageSendTask::SenderId).not_null())
                    .col(string(BackstageSendTask::SenderAccount).not_null())
                    .col(string_null(BackstageSendTask::SenderIp))
                    .col(integer(BackstageSendTask::ReceiverCount).not_null())
                    .col(array_null(
                        BackstageSendTask::ReceiverId,
                        ColumnType::BigInteger,
                    ))
                    .col(array_null(
                        BackstageSendTask::ReceiverAccount,
                        ColumnType::String(StringLen::N(255)),
                    ))
                    .col(string(BackstageSendTask::TaskName).not_null())
                    .col(integer(BackstageSendTask::NotifyLevel).not_null())
                    .col(integer(BackstageSendTask::TaskStatus).not_null())
                    .col(text_null(BackstageSendTask::ErrorMessage))
                    .col(big_integer(BackstageSendTask::ClientEventId).not_null())
                    .col(timestamp(BackstageSendTask::CreateAt).default(Expr::current_timestamp()))
                    .col(timestamp(BackstageSendTask::UpdateAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        let sql = r#"
            Comment on table backstage_send_task is '後台發送任務表';
            Comment on column backstage_send_task.id is 'ID';
            Comment on column backstage_send_task.client_id is '客戶ID';
            Comment on column backstage_send_task.sender_id is '發送者ID';
            Comment on column backstage_send_task.sender_account is '發送者帳號';
            Comment on column backstage_send_task.sender_ip is '發送者IP';
            Comment on column backstage_send_task.receiver_count is '接收者數量';
            Comment on column backstage_send_task.receiver_account is '接收者帳號';
            Comment on column backstage_send_task.receiver_id is '接收者ID';
            Comment on column backstage_send_task.task_name is '任務名稱';
            Comment on column backstage_send_task.notify_level is '通知等級';
            Comment on column backstage_send_task.task_status is '任務狀態';
            Comment on column backstage_send_task.error_message is '錯誤訊息';
            Comment on column backstage_send_task.client_event_id is '客戶事件ID';
            Comment on column backstage_send_task.create_at is '創建時間';
            Comment on column backstage_send_task.update_at is '更新時間';
        "#;

        manager.get_connection().execute_unprepared(sql).await?;
        Ok(())
    }

    async fn add_update_trigger(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        let sql = r#"
            CREATE TRIGGER trigger_update_timestamp
            BEFORE UPDATE ON backstage_send_task
            FOR EACH ROW
            EXECUTE FUNCTION update_timestamp();
        "#;

        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }
}
