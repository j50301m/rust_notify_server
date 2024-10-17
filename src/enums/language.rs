use kgs_err::models::status::Status as KgsStatus;
use sea_orm::{entity::prelude::*, strum::Display};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Display)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum Language {
    #[strum(to_string = "UsEn")]
    UsEn = 0, // 美式英文
    #[strum(to_string = "Jp")]
    Jp = 1, // 日文
    #[strum(to_string = "ZhTw")]
    ZhTw = 2, // 繁體中文
    #[strum(to_string = "ZhCn")]
    ZhCn = 3, // 簡體中文
}

impl From<Language> for i32 {
    fn from(language: Language) -> Self {
        language as i32
    }
}

impl TryFrom<i32> for Language {
    type Error = KgsStatus;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Language::UsEn),
            1 => Ok(Language::Jp),
            2 => Ok(Language::ZhTw),
            3 => Ok(Language::ZhCn),
            _ => Err(KgsStatus::InvalidArgument),
        }
    }
}

impl Language {
    pub fn to_id(&self) -> i32 {
        self.clone().into()
    }
}
