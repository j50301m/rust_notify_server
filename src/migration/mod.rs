mod m20240618_001_create_update_trigger_function;
mod m20240619_001_create_failed_record;
mod m20240619_002_create_success_record;
mod m20240619_003_create_language;
mod m20240619_005_create_notify_template;
mod m20240619_006_create_notify_event;
mod m20240619_007_create_notify_type;
mod m20240619_008_create_notify_level;
mod m20240619_009_create_client_notify_template;
mod m20240619_010_create_client_notify_event;
mod m20240619_011_create_notify_records;
mod m20240619_012_create_notify_status;
mod m20240708_001_create_platform;
mod m20240710_001_create_task_status;
mod m20240710_002_create_backstage_send_task;
mod m20240710_003_create_backstage_send_task_detail;

pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240618_001_create_update_trigger_function::Migration), // 新增更新時間觸發器
            Box::new(m20240619_001_create_failed_record::Migration),           // 新增錯誤紀錄表
            Box::new(m20240619_002_create_success_record::Migration),          // 新增成功紀錄表
            Box::new(m20240619_003_create_language::Migration),                // 新增語言表
            Box::new(m20240619_005_create_notify_template::Migration),         // 新增模板表
            Box::new(m20240619_006_create_notify_event::Migration),            // 新增事件類型表
            Box::new(m20240619_007_create_notify_type::Migration),             // 新增通知類型表
            Box::new(m20240619_008_create_notify_level::Migration),            // 新增通知等級表
            Box::new(m20240619_009_create_client_notify_template::Migration),  // 新增客戶通知模板表
            Box::new(m20240619_010_create_client_notify_event::Migration),     // 新增客戶通知事件表
            Box::new(m20240619_011_create_notify_records::Migration),          // 新增通知紀錄表
            Box::new(m20240619_012_create_notify_status::Migration),           // 新增通知狀態表
            Box::new(m20240708_001_create_platform::Migration),                // 新增通知平台表
            Box::new(m20240710_001_create_task_status::Migration),             // 新增任務狀態表
            Box::new(m20240710_002_create_backstage_send_task::Migration),     // 新增後台發送任務表
            Box::new(m20240710_003_create_backstage_send_task_detail::Migration), // 新增後台發送任務詳情表
        ]
    }
}
