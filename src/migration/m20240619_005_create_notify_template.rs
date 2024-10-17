use std::vec;

use crate::entity::notify_template;
use crate::enums;
use sea_orm::{EntityTrait, PaginatorTrait, Set};
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
            .drop_table(Table::drop().table(NotifyTemplate::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum NotifyTemplate {
    Table,
    Id,
    NotifyLevel,
    NotifyEvent,
    LanguageId,
    KeyList,
    NotifyType,
    Title,
    Content,
    IsEditable,
}

impl Migration {
    async fn create_table(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NotifyTemplate::Table)
                    .if_not_exists()
                    .col(
                        big_integer(NotifyTemplate::Id)
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(integer(NotifyTemplate::NotifyLevel).not_null())
                    .col(integer(NotifyTemplate::NotifyEvent).not_null())
                    .col(integer(NotifyTemplate::LanguageId).not_null())
                    .col(array_null(
                        NotifyTemplate::KeyList,
                        ColumnType::String(StringLen::N(50)),
                    ))
                    .col(integer(NotifyTemplate::NotifyType).not_null())
                    .col(text(NotifyTemplate::Title).not_null())
                    .col(text(NotifyTemplate::Content).not_null())
                    .col(boolean(NotifyTemplate::IsEditable).default(true))
                    .to_owned(),
            )
            .await?;

        let sql = r#"
            Comment on column notify_template.notify_event is '通知事件類型 ID ex: ㄧ般通知、登入異常、註冊成功...';
            Comment on column notify_template.notify_level is '通知等級 Id';
            Comment on column notify_template.language_id is '語言 ID';
            Comment on column notify_template.key_list is '挖空的關鍵字列表 ex: {{verify_code}} , {{username}}';
            Comment on column notify_template.notify_type is '發送類型: 1: 站內信, 2: Email, 3: SMS';
            Comment on column notify_template.title is '標題';
            Comment on column notify_template.content is '內容';
            Comment on column notify_template.is_editable is '是否可編輯';
        "#;

        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }

    async fn create_initial_data(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // if table is empty, insert initial data
        if notify_template::Entity::find().count(db).await? > 0 {
            return Ok(());
        }

        let data = vec![
            notify_template::ActiveModel {
                id: Set(1),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::LoginAnomaly),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(
                    vec![
                        "{{last_country_code}}".to_string(),
                        "{{now_country_code}}".to_string(),
                        "{{last_city}}".to_string(),
                        "{{now_city}}".to_string(),
                        "{{last_ip}}".to_string(),
                        "{{now_ip}}".to_string(),
                        "{{now_browser}}".to_string(),
                        "{{last_browser}}".to_string(),
                        "{{now_os}}".to_string(),
                        "{{last_os}}".to_string(),
                        "{{login_result}}".to_string(),
                    ])),
                notify_type: Set(enums::NotifyType::InApp),
                title: Set("登入異常通知".to_string()),
                content:Set("偵測到你的帳號登入異常, 上次登入IP: {{last_ip}} 國家(代號):{{last_country_code}} 城市: {{last_city}} , 本次登入IP: {{now_ip}} 區域:{{now_country_code}} 國家:{{now_city}}".to_string()),
                is_editable: Set(true),
            },
            notify_template::ActiveModel {
                id: Set(2),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::RegisterSuccess),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                notify_type: Set(enums::NotifyType::InApp),
                title: Set("註冊成功通知".to_string()),
                content:Set("誠摯歡迎 貴賓:{{user_account}} 加入Kgs測試平台".to_string()),
                is_editable: Set(true),
            },
            notify_template::ActiveModel {
                id: Set(3),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::DepositSuccess),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(vec!["{{from_amount}}".to_string(),"{{from_currency}}".to_string(),"{{to_amount}}".to_string(),"{{to_currency}}".to_string(),"{{transaction_id}}".to_string()])),
                notify_type: Set(enums::NotifyType::InApp),
                title: Set("轉帳成功通知".to_string()),
                content:Set("您已成功轉帳: 交易單號：{{transaction_id}} 轉帳金額： {{from_currency}} {{from_amount}} 入帳金額 {{to_currency}} {{to_amount}}".to_string()),
                is_editable: Set(true),
            },
            notify_template::ActiveModel {
                id: Set(4),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::WithdrawSuccess),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(vec!["{{from_amount}}".to_string(),"{{from_currency}}".to_string(),"{{to_amount}}".to_string(),"{{to_currency}}".to_string(),"{{transaction_id}}".to_string()])),
                notify_type: Set(enums::NotifyType::InApp),
                title: Set("提款成功通知".to_string()),
                content:Set("您已成功取款: 交易單號：{{transaction_id}} 從平台轉出金額： {{from_currency}} {{from_amount}} 入帳金額 {{to_currency}} {{to_amount}}".to_string()),
                is_editable: Set(true),
            },
            notify_template::ActiveModel {
                id: Set(5),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::WithdrawFail),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(vec!["{{transaction_id}}".to_string()])),
                notify_type: Set(enums::NotifyType::InApp),
                title: Set("出金失敗".to_string()),
                content:Set("出金失敗: 交易單號:{{transaction_id}}".to_string()),
                is_editable: Set(true),
            },
            notify_template::ActiveModel {
                id: Set(6),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::KycVerifySuccess),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                notify_type: Set(enums::NotifyType::InApp),
                title: Set("Kyc通過".to_string()),
                content:Set("您的KYC認證已通過".to_string()),
                is_editable: Set(true),
            },
            notify_template::ActiveModel {
                id: Set(7),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::KycVerifyFail),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(vec!["{{reason}}".to_string()])),
                notify_type: Set(enums::NotifyType::InApp),
                title: Set("Kyc失敗".to_string()),
                content:Set("您的KYC認證失敗".to_string()),
                is_editable: Set(true),
            },
            notify_template::ActiveModel {
                id: Set(8),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::KycReverify),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                notify_type: Set(enums::NotifyType::InApp),
                title: Set("Kyc重新驗證".to_string()),
                content:Set("您的KYC認證已重新驗證".to_string()),
                is_editable: Set(true),
            },
            notify_template::ActiveModel {
                id: Set(9),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::CreditCardVerifySuccess),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                notify_type: Set(enums::NotifyType::InApp),
                title: Set("信用卡驗證成功".to_string()),
                content:Set("您的信用卡已驗證成功".to_string()),
                is_editable: Set(true),
            },
            notify_template::ActiveModel {
                id: Set(10),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::CreditCardVerifyFail),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                notify_type: Set(enums::NotifyType::InApp),
                title: Set("信用卡驗證失敗".to_string()),
                content:Set("您的信用卡驗證失敗".to_string()),
                is_editable: Set(true),
            },
            notify_template::ActiveModel {
                id: Set(11),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::LoginWarning),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(vec!["{{failed_times}}".to_string()])),
                notify_type: Set(enums::NotifyType::Email),
                title: Set("登入警告".to_string()),
                content:Set("密碼錯誤次數超過 {{failed_times}} 故將您的帳號鎖住 以保安全".to_string()),
                is_editable: Set(true),
            },
            notify_template::ActiveModel {
                id: Set(12),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::UpdateProfileSuccess),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                notify_type: Set(enums::NotifyType::InApp),
                title: Set("更新個人資料成功".to_string()),
                content:Set("您的個人資料已更新成功".to_string()),
                is_editable: Set(true),
            },
            notify_template::ActiveModel {
                id: Set(13),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::LoginSuccess),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                notify_type: Set(enums::NotifyType::Email),
                title: Set("登入成功".to_string()),
                content:Set("您已成功登入".to_string()),
                is_editable: Set(true),
            },
            notify_template::ActiveModel {
                id: Set(14),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::VerifyEmail),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(vec!["{{verify_code}}".to_string()])),
                notify_type: Set(enums::NotifyType::Email),
                title: Set("信箱驗證".to_string()),
                content:Set("您的驗證碼{{verify_code}}".to_string()),
                is_editable: Set(true),
            },
            notify_template::ActiveModel {
                id: Set(15),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::VerifyPhone),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(vec!["{{verify_code}}".to_string()])),
                notify_type: Set(enums::NotifyType::SMS),
                title: Set("手機號碼驗證".to_string()),
                content:Set("您的驗證碼{{verify_code}}".to_string()),
                is_editable: Set(true),
            },
            notify_template::ActiveModel {
                id: Set(16),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::BackstageVerifyKyc),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                notify_type: Set(enums::NotifyType::InApp),
                title: Set("{{user_account}} KYC審核".to_string()),
                content:Set("{{user_account}}申請Kyc驗證".to_string()),
                is_editable: Set(true),
            },
            notify_template::ActiveModel {
                id: Set(17),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::BackstageVerifyWithdraw),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                notify_type: Set(enums::NotifyType::InApp),
                title: Set("{{user_account}} 出金審核".to_string()),
                content: Set("{{user_account}}申請出金".to_string()),
                is_editable: Set(true),
            },
            notify_template::ActiveModel {
                id:Set(18),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::BackstageVerifyDeposit),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                notify_type: Set(enums::NotifyType::InApp),
                title: Set("{{user_account}} 入金審核".to_string()),
                content: Set("{{user_account}}申請入金".to_string()),
                is_editable: Set(true),
            },
            notify_template::ActiveModel {
                id: Set(19),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::VipLevelUp),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(vec!["{{new_vip_level}}".to_string(),"{{old_vip_level}}".to_string()])),
                notify_type: Set(enums::NotifyType::InApp),
                title: Set("{{user_account}} 等級提升".to_string()),
                content: Set("恭喜由{{old_vip_level}} 提升至{{new_vip_level}}".to_string()),
                is_editable: Set(true),
            },
            notify_template::ActiveModel {
                id: Set(20),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::LoginPasswordReset),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(vec!["{{password}}".to_string()])),
                notify_type: Set(enums::NotifyType::Email),
                title: Set("密碼重置".to_string()),
                content: Set("您的密碼已重置為{{password}}".to_string()),
                is_editable: Set(true),
            },
            notify_template::ActiveModel {
                id: Set(21),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::WithdrawPasswordReset),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                notify_type: Set(enums::NotifyType::InApp),
                title: Set("取款密碼重置".to_string()),
                content: Set("您的取款密碼已重置  請儘速設置新的密碼".to_string()),
                is_editable: Set(true),
            },
            notify_template::ActiveModel {
                id: Set(22),
                notify_level: Set(enums::NotifyLevel::System),
                notify_event: Set(enums::NotifyEvent::LoginPasswordChange),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                notify_type: Set(enums::NotifyType::InApp),
                title: Set("登入密碼修改".to_string()),
                content: Set("您的登入密碼已經修改".to_string()),
                is_editable: Set(true),
            },
        ];

        notify_template::Entity::insert_many(data).exec(db).await?;

        Ok(())
    }
}
