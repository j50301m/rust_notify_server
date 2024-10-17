use std::vec;

use chrono::Utc;
use sea_orm::ActiveValue::NotSet;
use sea_orm::{EntityTrait, PaginatorTrait, Set};
use sea_orm_migration::{prelude::*, schema::*};

use crate::entity::client_notify_template;
use crate::enums;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        self.create_table(manager).await?;

        self.create_initial_data(manager).await?;

        self.add_update_trigger(manager).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ClientNotifyTemplate::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ClientNotifyTemplate {
    Table,
    Id,
    ClientId,
    ClientNotifyEvent,
    LanguageId,
    KeyList,
    NotifyType,
    Title,
    Content,
    IsSystem,
    CreateAt,
    UpdateAt,
}

impl Migration {
    async fn create_table(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ClientNotifyTemplate::Table)
                    .if_not_exists()
                    .col(
                        big_integer(ClientNotifyTemplate::Id)
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(big_integer(ClientNotifyTemplate::ClientId).not_null())
                    .col(big_integer(ClientNotifyTemplate::ClientNotifyEvent).not_null())
                    .col(integer(ClientNotifyTemplate::LanguageId).not_null())
                    .col(array_null(
                        ClientNotifyTemplate::KeyList,
                        ColumnType::String(StringLen::N(50)),
                    ))
                    .col(integer(ClientNotifyTemplate::NotifyType).not_null())
                    .col(text(ClientNotifyTemplate::Title))
                    .col(text(ClientNotifyTemplate::Content))
                    .col(boolean(ClientNotifyTemplate::IsSystem).default(false))
                    .col(
                        timestamp(ClientNotifyTemplate::CreateAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp(ClientNotifyTemplate::UpdateAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        let sql = r#"
            Comment on table client_notify_template is '客戶端通知模板表';
            Comment on column client_notify_template.id is 'ID';
            Comment on column client_notify_template.client_id is '客戶端ID';
            Comment on column client_notify_template.client_notify_event is 'client設定的通知事件';
            Comment on column client_notify_template.language_id is '語言ID';
            Comment on column client_notify_template.key_list is '關鍵字列表';
            Comment on column client_notify_template.notify_type is '通知類型';
            Comment on column client_notify_template.title is '標題';
            Comment on column client_notify_template.content is '內容';
            Comment on column client_notify_template.is_system is '是否為系統訊息模板';
            Comment on column client_notify_template.create_at is '創建時間';
            Comment on column client_notify_template.update_at is '更新時間';
        "#;

        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }

    async fn create_initial_data(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // if table is empty, insert initial data
        if client_notify_template::Entity::find().count(db).await? > 0 {
            return Ok(());
        }

        let data = vec![
            client_notify_template::ActiveModel {
                id:NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::LoginAnomaly.to_id()),
                notify_type: Set(enums::NotifyType::Email),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(vec![
                "{{last_country_code}}".to_string(),
                "{{last_city}}".to_string(),
                "{{last_login_time}}".to_string(),
                "{{last_ip}}".to_string(),
                "{{now_country_code}}".to_string(),
                "{{now_city}}".to_string(),
                "{{now_login_time}}".to_string(),
                "{{now_ip}}".to_string(),
                "{{now_browser}}".to_string(),
                "{{last_browser}}".to_string(),
                "{{now_os}}".to_string(),
                "{{last_os}}".to_string(),
                "{{login_result}}".to_string(),
                ])),
                title: Set("異常登入通知".to_string()),
                content: Set(r#"
                <html>
                    <body>
                        <div>感謝您對 Kgs測試 平台的支持。</div>
                        <div>我們希望提供給您一個安全可靠的遊戲環境，因此我們特意提醒您最近登入成功紀錄如下，供您參考：</div>
                        <p>&nbsp;</p>
                        <div>登入詳細資料</div>
                        <div>登入時間　　　　 {{now_login_time}}</div>
                        <div>登入結果　　　　 {{login_result}}</div>
                        <div>登入ＩＰ　　　　 {{now_ip}}</div>
                        <div>登入地點　　　　 {{now_country_code}} {{now_city}}</div>
                        <div>登入瀏覽器　　　　　 {{now_browser}}</div>
                        <div>作業系統　　　　{{now_os}}</div>
                        <p>&nbsp;</p>
                        <div>根據我們的紀錄，您最近的登入位置與您通常的習慣有所不同。這可能是因為您正在旅行，或者您使用了新的設備或瀏覽器進行了登錄。</div>
                        <div>如果這是您本人的操作，請忽略此訊息。但如果您認為這次登入不是由您本人執行，我們建議您立即更改您的密碼，以確保您的帳號安全。</div>
                        <p>&nbsp;</p>
                        <div>如果您對這次登入有任何疑問，或者需要任何協助，請隨時聯繫我們的24小時線上客服團隊。我們將竭誠為您提供幫助與支援。</div>
                        <p>&nbsp;</p>
                        <div>再次感謝您對 {{brand_name}} 平台的支持與信任。祝您遊戲愉快！</div>
                        <p>&nbsp;</p>
                        <div>登入網站即刻暢玩 http://www.test124.com</div>
                        <div>聯繫在線客服 http://www.test124.com</div>
                    </body>
                </html>
                "#.to_string()),
                is_system: Set(true),
                create_at:Set(Utc::now().naive_utc()),
                update_at:Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::RegisterSuccess.to_id()),
                notify_type: Set(enums::NotifyType::InApp),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                title: Set("感謝您的註冊！".to_string()),
                content: Set(r#"
                    非常感謝您成為我們的會員，我們期待為您提供最暢快的遊戲體驗！無論您喜歡哪種類型的遊戲，我們都希望能讓您在我們的平台上找到您喜愛的遊戲。
                    現在，您可以前往我們的活動頁面，了解我們最新的優惠活動、獎勵計劃和促銷活動。無論您是新手還是老手，我們都有適合您的活動和獎勵等您來參與！
                    祝您遊戲愉快！
                    前往活動頁面 http://www.test124.com
                "#.to_string()),
                is_system: Set(true),
                create_at:Set(Utc::now().naive_utc()),
                update_at:Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::RegisterSuccess.to_id()),
                notify_type: Set(enums::NotifyType::Email),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                title: Set("感謝您的註冊！".to_string()),
                content: Set(r#"
                    <html>
                        <div>標題：感謝您的註冊！</div>
                        <div>非常感謝您成為我們的會員，我們期待為您提供最暢快的遊戲體驗！無論您喜歡哪種類型的遊戲，我們都希望能讓您在我們的平台上找到您喜愛的遊戲。</div>
                        <div>現在，您可以前往我們的活動頁面，了解我們最新的優惠活動、獎勵計劃和促銷活動。無論您是新手還是老手，我們都有適合您的活動和獎勵等您來參與！</div>
                        <div>祝您遊戲愉快！</div>
                        <div>登入網站即刻暢玩 http://www.test123.com</div>
                        <div>前往活動頁面 http://www.test123.com</div>
                    </html>
                "#.to_string()),
                is_system: Set(true),
                create_at:Set(Utc::now().naive_utc()),
                update_at:Set(Utc::now().naive_utc()),
            },

            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::DepositSuccess.to_id()),
                notify_type: Set(enums::NotifyType::InApp),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(vec![
                    "{{from_amount}}".to_string(),
                    "{{from_currency}}".to_string(),
                    "{{to_currency}}".to_string(),
                    "{{to_amount}}".to_string(),
                    "{{transaction_id}}".to_string(),
                    ])),
                title: Set("入金成功".to_string()),
                content: Set(r#"
                {{user_account}}
                    您好，我們很高興通知您，您的存款金額已成功存入您的錢包中。請您刷新頁面查看最新餘額。
                    祝您在我們的平台上遊戲愉快，有任何問題請隨時聯繫我們的客服團隊。
                    交易單號：{{transaction_id}}
                    金額：{{to_amount}}
                "#.to_string()),
                is_system: Set(true),
                create_at:Set(Utc::now().naive_utc()),
                update_at:Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::WithdrawSuccess.to_id()),
                notify_type: Set(enums::NotifyType::InApp),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(vec![
                    "{{from_amount}}".to_string(),
                    "{{from_currency}}".to_string(),
                    "{{to_currency}}".to_string(),
                    "{{to_amount}}".to_string(),
                    "{{transaction_id}}".to_string(),
                    ])),
                title: Set("出金成功".to_string()),
                content: Set(r#"
                    {{user_account}}
                    您好，我們很高興地通知您，您的出款申請已成功處理！請注意，提現金額可能需要一些時間才能到帳，我們建議您耐心等候。
                    如果您對出款有任何疑問或需要進一步的幫助，請隨時聯繫我們的線上客服，我們將竭誠為您服務。感謝您的支持與配合。
                    交易單號：{{transaction_id}}
                    金額：{{to_amount}}
                "#.to_string()),
                is_system: Set(true),
                create_at:Set(Utc::now().naive_utc()),
                update_at:Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::WithdrawFail.to_id()),
                notify_type: Set(enums::NotifyType::InApp),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(vec![
                    "{{transaction_id}}".to_string(),
                ])),
                title: Set("出金失敗".to_string()),
                content: Set(r#"
                    {{user_account}}
                    您好，我們很抱歉地通知您，您的出款申請已被系統取消。請您刷新頁面確認餘額。
                    交易單號：{{transaction_id}}

                    如果您對此有任何疑問或需要進一步的解釋，請立即聯繫我們的線上客服，我們將竭誠為您服務。感謝您的理解與合作。

                    [拒絕原因]
                    1.未完成遊戲要求：遊戲有特定的投注要求，如必須達到一定的投注額或遊戲回合數才能取款。
                    2.違反平台規則：您可能在遊戲中違反了平台的規則，如使用不正當的方法進行遊戲或有違反規定的行為。
                    3.資料不完整或錯誤：您提供的個人資料不完整或有錯誤時，請重新提供更正確的資訊。
                    4.安全審核：為確保資金安全，需要進行安全審核。在這種情況下，可能需要您提供額外的身份證明文件或進行進一步的驗證。
                    5.技術問題：有時系統或支付平台可能出現技術問題。
                    6.收款帳戶被限制：您提供的收款帳戶可能因各種原因而受到限制，如帳戶凍結、支付服務商限制或其他相關問題。
                    7.銀行維護：收款或出款的銀行可能因例行維護、系統升級或其他技術問題而暫時無法處理交易。
                    8.其他原因
                "#.to_string()),
                is_system: Set(true),
                create_at:Set(Utc::now().naive_utc()),
                update_at:Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::KycVerifySuccess.to_id()),
                notify_type: Set(enums::NotifyType::InApp),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                title: Set("身份驗證認證完成通知".to_string()),
                content: Set(r#"
                    {{user_account}}
                    您好，我們很高興地通知您，您的身份驗證已經成功完成！從現在開始，您可以自由地申請提款。
                "#.to_string()),
                is_system: Set(true),
                create_at:Set(Utc::now().naive_utc()),
                update_at:Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::KycVerifySuccess.to_id()),
                notify_type: Set(enums::NotifyType::Email),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                title: Set("身份驗證認證完成通知".to_string()),
                content: Set(r#"
                    <html>
                        <div>
                        <div>{{user_account}}</div>
                        <div>您好，我們很高興地通知您，您的身份驗證已經成功完成！從現在開始，您可以自由地申請提款。</div>
                        </div>
                    </html>
                "#.to_string()),
                is_system: Set(true),
                create_at:Set(Utc::now().naive_utc()),
                update_at:Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::KycVerifyFail.to_id()),
                notify_type: Set(enums::NotifyType::InApp),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(vec![
                        "{{reason}}".to_string(),
                ])),
                title: Set("身份驗證認證失敗通知".to_string()),
                content: Set(r#"
                    {{user_account}}
                    您好，我們很抱歉地通知您，系統對您提交的身份認證文件進行了核驗，發現存在不正確、缺失或不可讀取的情況。因此，我們無法完成您的身份驗證。 

                    請您再次核對您提交的文件內容，確保文件的完整性和正確性，然後重新提交。對於由此給您帶來的任何不便，我們深感抱歉。

                    以下是一些可能的拒絕原因及解決方法：
                    [請您再次提交詳細相關文件]

                    身份證明無效。

                    1.系統檢測到您個人資料中的全名/出生日期不正確、缺失或不可讀取，請確認您的全名、出生日期和有效期可以清晰易讀。
                    2.上傳的文件中沒有您的臉部照片或您的臉部照片不清晰，請上傳臉部特徵需要清晰展示。
                    3.提供的地址證明顯示的是另一個人的姓名，而不是您的姓名
                    4.您提交的證明文件不可被接受。
                    5.上傳文件中缺少了部分頁面。
                    6.上傳文件質量差或有損壞，請上傳一張可見的新的文件四角的新文件照片（包括正面和反面）。確保文檔頁面中重要的詳細信息的頁面都有拍攝到。
                    7.上傳的身份證件已過期，請確保身份證件仍在有效期內且未過期。

                    地址證明無效。

                    1.您提交的證明資料不可被接受，請您提交[1張圖片]，圖片內容包含「姓名、地址、發行日期（3個月以內）」可以核實資料的相關文件（政府簽發的居住證、電費、水費帳單等）。
                    2.上傳圖片中缺少了部分頁面。
                    如果您希望重新提交或新增其他文件，請在「其他證明文件」部分上傳。

                    [缺失相關資訊]

                    基本資料未填寫。

                    很抱歉地通知您，由於您的基本資訊尚未填寫完整，我們無法進行身份驗證。為了確保您的帳戶安全並獲得更好的服務，請您儘快填寫完整基本資訊。

                    有關身份驗證所需文件的更多訊息，請參閱 (驗證須知頁面) 。

                    如果您對此有任何疑問或需要進一步的幫助，請隨時聯繫我們的線上客服，我們將竭誠為您服務。再次感謝您的支持與理解。
                "#.to_string()),
                is_system: Set(true),
                create_at:Set(Utc::now().naive_utc()),
                update_at:Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::KycVerifyFail.to_id()),
                notify_type: Set(enums::NotifyType::Email),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(vec![
                        "{{reason}}".to_string(),
                ])),
                title: Set("身份驗證認證失敗通知".to_string()),
                content: Set(r#"
                    <div>
                    <div>{{user_account}}</div>
                    <div>您好，我們很抱歉地通知您，系統對您提交的身份認證文件進行了核驗，發現存在不正確、缺失或不可讀取的情況。因此，我們無法完成您的身份驗證。</div>
                    <br />
                    <div>請您再次核對您提交的文件內容，確保文件的完整性和正確性，然後重新提交。對於由此給您帶來的任何不便，我們深感抱歉。</div>
                    <br />
                    <div>以下是一些可能的拒絕原因及解決方法：</div>
                    <div>[請您再次提交詳細相關文件]</div>
                    <br />
                    <div>身份證明無效。</div>
                    <br />
                    <div>1.系統檢測到您個人資料中的全名/出生日期不正確、缺失或不可讀取，請確認您的全名、出生日期和有效期可以清晰易讀。</div>
                    <div>2.上傳的文件中沒有您的臉部照片或您的臉部照片不清晰，請上傳臉部特徵需要清晰展示。</div>
                    <div>3.提供的地址證明顯示的是另一個人的姓名，而不是您的姓名</div>
                    <div>4.您提交的證明文件不可被接受。</div>
                    <div>5.上傳文件中缺少了部分頁面。</div>
                    <div>6.上傳文件質量差或有損壞，請上傳一張可見的新的文件四角的新文件照片（包括正面和反面）。確保文檔頁面中重要的詳細信息的頁面都有拍攝到。</div>
                    <div>7.上傳的身份證件已過期，請確保身份證件仍在有效期內且未過期。</div>
                    <br />
                    <div>地址證明無效。</div>
                    <br />
                    <div>1.您提交的證明資料不可被接受，請您提交[1張圖片]，圖片內容包含「姓名、地址、發行日期（3個月以內）」可以核實資料的相關文件（政府簽發的居住證、電費、水費帳單等）。</div>
                    <div>2.上傳圖片中缺少了部分頁面。</div>
                    <div>如果您希望重新提交或新增其他文件，請在「其他證明文件」部分上傳。</div>
                    <br />
                    <div>[缺失相關資訊]</div>
                    <br />
                    <div>基本資料未填寫。</div>
                    <br />
                    <div>很抱歉地通知您，由於您的基本資訊尚未填寫完整，我們無法進行身份驗證。為了確保您的帳戶安全並獲得更好的服務，請您儘快填寫完整基本資訊。</div>
                    <br />
                    <div>有關身份驗證所需文件的更多訊息，請參閱 (驗證須知頁面) 。</div>
                    <br />
                    <div>如果您對此有任何疑問或需要進一步的幫助，請隨時聯繫我們的線上客服，我們將竭誠為您服務。再次感謝您的支持與理解。</div>
                    </div>
                "#.to_string()),
                is_system: Set(true),
                create_at:Set(Utc::now().naive_utc()),
                update_at:Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::KycReverify.to_id()),
                notify_type: Set(enums::NotifyType::InApp),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                title: Set("Kyc重新驗證".to_string()),
                content: Set("您的KYC認證已重新驗證".to_string()),
                is_system: Set(true),
                create_at:Set(Utc::now().naive_utc()),
                update_at:Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::KycReverify.to_id()),
                notify_type: Set(enums::NotifyType::Email),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                title: Set("Kyc重新驗證".to_string()),
                content: Set("您的KYC認證已重新驗證".to_string()),
                is_system: Set(true),
                create_at:Set(Utc::now().naive_utc()),
                update_at:Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::CreditCardVerifySuccess.to_id()),
                notify_type: Set(enums::NotifyType::InApp),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                title: Set("信用卡驗證成功".to_string()),
                content: Set("您的信用卡已驗證成功".to_string()),
                is_system: Set(true),
                create_at:Set(Utc::now().naive_utc()),
                update_at:Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::CreditCardVerifySuccess.to_id()),
                notify_type: Set(enums::NotifyType::Email),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                title: Set("信用卡驗證成功".to_string()),
                content: Set("您的信用卡已驗證成功".to_string()),
                is_system: Set(true),
                create_at:Set(Utc::now().naive_utc()),
                update_at:Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::CreditCardVerifyFail.to_id()),
                notify_type: Set(enums::NotifyType::InApp),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                title: Set("信用卡驗證失敗".to_string()),
                content: Set("您的信用卡驗證失敗".to_string()),
                is_system: Set(true),
                create_at:Set(Utc::now().naive_utc()),
                update_at:Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::CreditCardVerifyFail.to_id()),
                notify_type: Set(enums::NotifyType::Email),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                title: Set("信用卡驗證失敗".to_string()),
                content: Set("您的信用卡驗證失敗".to_string()),
                is_system: Set(true),
                create_at:Set(Utc::now().naive_utc()),
                update_at:Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::LoginWarning.to_id()),
                notify_type: Set(enums::NotifyType::Email),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(vec!["{{failed_times}}".to_string()])),
                title: Set("登入失敗次數過多".to_string()),
                content: Set("登入失敗次數已達{{failed_times}} 故將您的帳號鎖定 以保安全".to_string()),
                is_system: Set(true),
                create_at:Set(Utc::now().naive_utc()),
                update_at:Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::LoginSuccess.to_id()),
                notify_type: Set(enums::NotifyType::Email),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                title: Set("登入成功".to_string()),
                content: Set("{{user_account}} login success".to_string()),
                is_system: Set(true),
                create_at:Set(Utc::now().naive_utc()),
                update_at:Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::VerifyEmail.to_id()),
                notify_type: Set(enums::NotifyType::Email),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(
                    vec![
                        "{{verify_code}}".to_string(),
                    ]
                )),
                title: Set("驗證Email".to_string()),
                content: Set("
                {{user_account}} 您好！
                你得驗證碼是 {{verify_code}} ".to_string()),
                is_system: Set(true),
                create_at: Set(Utc::now().naive_utc()),
                update_at: Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::VerifyPhone.to_id()),
                notify_type: Set(enums::NotifyType::SMS),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(
                    vec![
                        "{{verify_code}}".to_string(),
                    ]
                )),
                title: Set("驗證手機".to_string()),
                content: Set("
                {{user_account}} 您好！
                你得驗證碼是 {{verify_code}} ".to_string()),
                is_system: Set(true),
                create_at: Set(Utc::now().naive_utc()),
                update_at: Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::UpdateProfileSuccess.to_id()),
                notify_type: Set(enums::NotifyType::InApp),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                title: Set("更新個人資訊成功".to_string()),
                content: Set("{{user_account}} 更新個人資訊成功".to_string()),
                is_system: Set(true),
                create_at: Set(Utc::now().naive_utc()),
                update_at: Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::VipLevelUp.to_id()),
                notify_type: Set(enums::NotifyType::InApp),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(
                    vec![
                        "{{new_vip_level}}".to_string(),
                        "{{old_vip_level}}".to_string(),
                    ]
                )),
                title: Set("恭喜vip等級提升".to_string()),
                content: Set("由VIP{{old_vip_level}} 升級到 VIP{{new_vip_level}} ".to_string()),
                is_system: Set(true),
                create_at: Set(Utc::now().naive_utc()),
                update_at: Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::VipLevelUp.to_id()),
                notify_type: Set(enums::NotifyType::Email),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(
                    vec![
                        "{{new_vip_level}}".to_string(),
                        "{{old_vip_level}}".to_string(),
                    ]
                )),
                title: Set("恭喜vip等級提升".to_string()),
                content: Set("由VIP{{old_vip_level}} 升級到 VIP{{new_vip_level}} ".to_string()),
                is_system: Set(true),
                create_at: Set(Utc::now().naive_utc()),
                update_at: Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::LoginPasswordReset.to_id()),
                notify_type: Set(enums::NotifyType::Email),
                language_id: Set(enums::Language::Jp),
                key_list: Set(Some(
                    vec![
                        "{{password}}".to_string(),
                    ]
                )),
                title: Set("登入密碼重置".to_string()),
                content: Set("新密碼 {{password}}".to_string()),
                is_system: Set(true),
                create_at: Set(Utc::now().naive_utc()),
                update_at: Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::WithdrawPasswordReset.to_id()),
                notify_type: Set(enums::NotifyType::Email),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                title: Set("出金密碼重置".to_string()),
                content: Set("請盡速更新您的取款密碼".to_string()),
                is_system: Set(true),
                create_at: Set(Utc::now().naive_utc()),
                update_at: Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::WithdrawPasswordReset.to_id()),
                notify_type: Set(enums::NotifyType::InApp),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                title: Set("出金密碼重置".to_string()),
                content: Set("請盡速更新您的取款密碼".to_string()),
                is_system: Set(true),
                create_at: Set(Utc::now().naive_utc()),
                update_at: Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::LoginPasswordChange.to_id()),
                notify_type: Set(enums::NotifyType::InApp),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                title: Set("密碼變更成功通知".to_string()),
                content: Set("{{user_account}}您好，我們想提醒您，您已成功進行密碼變更。為了保障您帳戶的資金安全，請您確認此次變更是否為本人操作。
                            如果您並未執行此操作，請立即聯繫我們的在線客服團隊，我們將協助您進一步處理。感謝您的支持與理解。".to_string()),
                is_system: Set(true),
                create_at: Set(Utc::now().naive_utc()),
                update_at: Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::LoginPasswordChange.to_id()),
                notify_type: Set(enums::NotifyType::Email),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                title: Set("密碼變更成功通知".to_string()),
                content: Set("{{user_account}}您好，我們想提醒您，您已成功進行密碼變更。為了保障您帳戶的資金安全，請您確認此次變更是否為本人操作。
                            如果您並未執行此操作，請立即聯繫我們的在線客服團隊，我們將協助您進一步處理。感謝您的支持與理解。".to_string()),
                is_system: Set(true),
                create_at: Set(Utc::now().naive_utc()),
                update_at: Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::WithdrawPasswordSet.to_id()),
                notify_type: Set(enums::NotifyType::Email),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                title: Set("支付密碼設置成功".to_string()),
                content: Set("{{user_account}}您的支付密碼已成功設置完成！這將有助於保障您的帳戶和交易的安全性。
                                請確保您妥善保存您的支付密碼，並勿與他人分享。這是確保您帳戶安全的關鍵。
                                如果您覺得有任何異常或有疑慮，請立即聯繫我們的在線客服團隊。我們隨時為您提供協助，並確保您的帳戶得到充分的保護。".to_string()),
                is_system: Set(true),
                create_at: Set(Utc::now().naive_utc()),
                update_at: Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135148985370546176),
                client_notify_event: Set(enums::NotifyEvent::WithdrawPasswordSet.to_id()),
                notify_type: Set(enums::NotifyType::InApp),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                title: Set("支付密碼設置成功".to_string()),
                content: Set("{{user_account}}您的支付密碼已成功設置完成！這將有助於保障您的帳戶和交易的安全性。
                                請確保您妥善保存您的支付密碼，並勿與他人分享。這是確保您帳戶安全的關鍵。
                                如果您覺得有任何異常或有疑慮，請立即聯繫我們的在線客服團隊。我們隨時為您提供協助，並確保您的帳戶得到充分的保護。".to_string()),
                is_system: Set(true),
                create_at: Set(Utc::now().naive_utc()),
                update_at: Set(Utc::now().naive_utc()),
            },


            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135149007982039040),
                client_notify_event: Set(enums::NotifyEvent::BackstageVerifyKyc.to_id()),
                notify_type: Set(enums::NotifyType::InApp),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                title: Set("{{user_account}}驗證KYC".to_string()),
                content: Set("{{user_account}}申請驗證KYC".to_string()),
                is_system: Set(true),
                create_at: Set(Utc::now().naive_utc()),
                update_at: Set(Utc::now().naive_utc()),
            },
            client_notify_template::ActiveModel {
                id: NotSet,
                client_id: Set(7135149007982039040),
                client_notify_event: Set(enums::NotifyEvent::BackstageVerifyWithdraw.to_id()),
                notify_type: Set(enums::NotifyType::InApp),
                language_id: Set(enums::Language::Jp),
                key_list: Set(None),
                title: Set("{{user_account}}申請提款".to_string()),
                content: Set("{{user_account}}申請提款".to_string()),
                is_system: Set(true),
                create_at: Set(Utc::now().naive_utc()),
                update_at: Set(Utc::now().naive_utc()),
            }
        ];

        client_notify_template::Entity::insert_many(data)
            .exec(db)
            .await?;

        Ok(())
    }

    async fn add_update_trigger(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        let sql = r#"
            CREATE TRIGGER trigger_update_timestamp
            BEFORE UPDATE ON client_notify_template
            FOR EACH ROW
            EXECUTE FUNCTION update_timestamp();
        "#;

        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }
}
