use kgs_err::models::status::Status as KgsStatus;
use sea_orm::{entity::prelude::*, strum::Display};
use serde::{Deserialize, Serialize};
/// 通知等級
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Display, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum NotifyLevel {
    #[strum(to_string = "Info")]
    Info = 1, // 一般通知
    #[strum(to_string = "System")]
    System = 2, // 系統通知
    #[strum(to_string = "Important")]
    Important = 3, // 重要通知
}

impl From<NotifyLevel> for i32 {
    fn from(notify_level: NotifyLevel) -> Self {
        notify_level as i32
    }
}

impl TryFrom<i32> for NotifyLevel {
    type Error = KgsStatus;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(NotifyLevel::Info),
            2 => Ok(NotifyLevel::System),
            3 => Ok(NotifyLevel::Important),
            _ => Err(KgsStatus::InvalidArgument),
        }
    }
}

impl NotifyLevel {
    pub fn to_id(&self) -> i32 {
        self.clone().into()
    }

    pub fn get_comment(&self) -> String {
        match self {
            NotifyLevel::Info => "一般通知",
            NotifyLevel::System => "系統通知",
            NotifyLevel::Important => "重要通知",
        }
        .to_string()
    }
}
