use crate::enums;
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "notify_template")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64, // 通知模板 ID
    pub notify_level: enums::NotifyLevel, // 通知等級 Id
    pub notify_event: enums::NotifyEvent, // 通知事件類型 ID ex: ㄧ般通知、登入異常、註冊成功...
    pub language_id: enums::Language,     // 語言 Id
    pub key_list: Option<Vec<String>>,    // 挖空的關鍵字列表 ex: {{verify_code}} , {{username}}
    pub notify_type: enums::NotifyType,   // 發送類型: 1: 站內信, 2: Email, 3: SMS
    pub title: String,                    // 標題
    pub content: String,                  // 內容
    pub is_editable: bool,                // 是否可編輯
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Language,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Language => Entity::belongs_to(super::language::Entity)
                .from(Column::LanguageId)
                .to(super::language::Column::Id)
                .into(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
