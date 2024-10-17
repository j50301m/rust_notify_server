use chrono::Utc;
use sea_orm::{ColumnTrait, EntityTrait, Iterable, QueryFilter, Set};
use sea_orm_migration::{prelude::*, schema::*};

use crate::entity::client_notify_event;
use crate::enums;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        self.create_table(manager).await?;

        self.create_initial_data(manager).await?;

        self.add_update_trigger(manager).await?;

        self.create_when_template_update_function(manager).await?;

        self.add_when_template_update_trigger(manager).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ClientNotifyEvent::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ClientNotifyEvent {
    Table,
    Id,
    ClientId,
    Platform,
    Name,
    Memo,
    IsSystemEvent,
    NotifyTypes,
    EditorAccount,
    CreateAt,
    UpdateAt,
}

impl Migration {
    async fn create_table(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ClientNotifyEvent::Table)
                    .if_not_exists()
                    .col(big_integer(ClientNotifyEvent::Id))
                    .col(big_integer(ClientNotifyEvent::ClientId))
                    .col(integer(ClientNotifyEvent::Platform).not_null())
                    .col(string(ClientNotifyEvent::Name).not_null())
                    .col(string(ClientNotifyEvent::Memo).not_null())
                    .col(
                        boolean(ClientNotifyEvent::IsSystemEvent)
                            .not_null()
                            .default(false),
                    )
                    .col(array_null(
                        ClientNotifyEvent::NotifyTypes,
                        ColumnType::Integer,
                    ))
                    .col(
                        timestamp(ClientNotifyEvent::CreateAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp(ClientNotifyEvent::UpdateAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(string(ClientNotifyEvent::EditorAccount).not_null())
                    .primary_key(
                        Index::create()
                            .col(ClientNotifyEvent::Id)
                            .col(ClientNotifyEvent::ClientId),
                    )
                    .to_owned(),
            )
            .await?;

        let sql = r#"
            Comment on table client_notify_event is '客戶端的通知事件';
            Comment on column client_notify_event.id is '事件ID';
            Comment on column client_notify_event.client_id is '客戶端ID';
            Comment on column client_notify_event.platform is '此事件通知通知平台 1.前台 2.後台 3.總管理後台';
            Comment on column client_notify_event.name is '事件名稱';
            Comment on column client_notify_event.memo is '事件描述';
            Comment on column client_notify_event.notify_types is '此事件支援的通知類型';
            Comment on column client_notify_event.editor_account is '編輯者帳號';
            Comment on column client_notify_event.is_system_event is '判別是不是系統預設事件 如果是則不可刪除與修改 id與name';
            Comment on column client_notify_event.create_at is '建立時間';
            Comment on column client_notify_event.update_at is '更新時間';
        "#;

        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }

    async fn create_initial_data(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let get_notify_type = |platform: enums::Platform| -> Option<Vec<enums::NotifyType>> {
            if platform == enums::Platform::Frontend {
                Some(vec![
                    enums::NotifyType::InApp,
                    enums::NotifyType::Email,
                    enums::NotifyType::SMS,
                ])
            } else {
                Some(vec![enums::NotifyType::InApp])
            }
        };

        for notify_event in enums::NotifyEvent::iter() {
            // Check if the notify_event exists
            if client_notify_event::Entity::find()
                .filter(client_notify_event::Column::Id.eq(notify_event.to_id()))
                .filter(client_notify_event::Column::ClientId.eq(7135148985370546176_i64))
                .one(db)
                .await?
                .is_some()
            {
                continue;
            }

            let client_id = match notify_event.get_platform() {
                enums::Platform::Frontend => 7135148985370546176,
                enums::Platform::Backstage => 7135149007982039040,
                _ => 0,
            };

            // insert the frontend system event into client_notify_event
            client_notify_event::Entity::insert(client_notify_event::ActiveModel {
                id: Set(notify_event.to_id()),
                client_id: Set(client_id),
                platform: Set(notify_event.get_platform()),
                name: Set(notify_event.to_string()),
                notify_types: Set(get_notify_type(notify_event.get_platform())),
                editor_account: Set("System".to_string()),
                memo: Set(notify_event.get_comment()),
                is_system_event: Set(true),
                create_at: Set(Utc::now().naive_utc()),
                update_at: Set(Utc::now().naive_utc()),
            })
            .exec(db)
            .await?;
        }

        Ok(())
    }

    async fn add_update_trigger(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        let sql = r#"
            CREATE TRIGGER trigger_update_timestamp
            BEFORE UPDATE ON client_notify_event
            FOR EACH ROW
            EXECUTE FUNCTION update_timestamp();
        "#;

        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }

    async fn create_when_template_update_function(
        &self,
        manager: &SchemaManager<'_>,
    ) -> Result<(), DbErr> {
        let sql = r#"
            CREATE OR REPLACE FUNCTION update_client_event_when_template_update()
            RETURNS TRIGGER AS $$
            BEGIN
                Update client_notify_event
                SET  update_at = now()
                WHERE client_notify_event.id = New.client_notify_event
                AND client_notify_event.client_id = New.client_id;
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;
        "#;

        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }

    async fn add_when_template_update_trigger(
        &self,
        manager: &SchemaManager<'_>,
    ) -> Result<(), DbErr> {
        let sql = r#"
            CREATE TRIGGER trigger_update_client_event_when_template_update
            AFTER UPDATE ON client_notify_template
            FOR EACH ROW
            EXECUTE FUNCTION update_client_event_when_template_update();
        "#;

        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }
}
