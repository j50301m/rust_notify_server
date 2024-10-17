use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        self.create_table(manager).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(BackstageSendTaskDetail::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum BackstageSendTaskDetail {
    Table,
    Id,
    BackstageSendTaskId,
    NotifyLevel,
    NotifyType,
    Title,
    Content,
}

impl Migration {
    async fn create_table(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BackstageSendTaskDetail::Table)
                    .if_not_exists()
                    .col(
                        big_integer(BackstageSendTaskDetail::Id)
                            .not_null()
                            .primary_key(),
                    )
                    .col(big_integer(BackstageSendTaskDetail::BackstageSendTaskId).not_null())
                    .col(integer(BackstageSendTaskDetail::NotifyType).not_null())
                    .col(integer(BackstageSendTaskDetail::NotifyLevel).not_null())
                    .col(string(BackstageSendTaskDetail::Title).not_null())
                    .col(text(BackstageSendTaskDetail::Content).not_null())
                    .to_owned(),
            )
            .await?;

        let sql = r#"
            Comment on table backstage_send_task_detail is '後台發送任務詳情表';
            Comment on column backstage_send_task_detail.id is 'ID';
            Comment on column backstage_send_task_detail.backstage_send_task_id is '後台發送任務ID';
            Comment on column backstage_send_task_detail.notify_type is '通知類型';
            Comment on column backstage_send_task_detail.notify_level is '通知等級';
            Comment on column backstage_send_task_detail.title is '標題';
            Comment on column backstage_send_task_detail.content is '內容';
        "#;

        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }
}
