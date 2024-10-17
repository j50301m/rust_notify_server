use kgs_err::models::status::Status as KgsStatus;
use sea_orm::{entity::prelude::*, strum::Display};
use serde::{Deserialize, Serialize};

/// 通知類型
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    EnumIter,
    DeriveActiveEnum,
    Display,
    Serialize,
    Deserialize,
    Hash,
)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum NotifyType {
    #[strum(to_string = "InApp")]
    InApp = 1, // 站內信
    #[strum(to_string = "Email")]
    Email = 2, // Email
    #[strum(to_string = "SMS")]
    SMS = 3, // SMS
}

impl From<NotifyType> for i32 {
    fn from(notify_type: NotifyType) -> Self {
        notify_type as i32
    }
}

impl TryFrom<i32> for NotifyType {
    type Error = KgsStatus;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(NotifyType::InApp),
            2 => Ok(NotifyType::Email),
            3 => Ok(NotifyType::SMS),
            _ => Err(KgsStatus::InvalidArgument),
        }
    }
}

impl NotifyType {
    pub fn to_id(&self) -> i32 {
        self.clone().into()
    }

    pub fn get_comment(&self) -> String {
        match self {
            NotifyType::InApp => "站內信",
            NotifyType::Email => "Email",
            NotifyType::SMS => "SMS",
        }
        .to_string()
    }
}
