use kgs_err::models::status::Status as KgsStatus;
use sea_orm::{entity::prelude::*, strum::Display};

/// 通知狀態
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Display)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum NotifyStatus {
    #[strum(to_string = "Unread")]
    Unread = 1, // 未讀
    #[strum(to_string = "Read")]
    Read = 2, // 已讀
    #[strum(to_string = "Delete")]
    Delete = 3, // 已刪除
}

impl From<NotifyStatus> for i32 {
    fn from(status: NotifyStatus) -> Self {
        status as i32
    }
}

impl TryFrom<i32> for NotifyStatus {
    type Error = KgsStatus;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(NotifyStatus::Unread),
            2 => Ok(NotifyStatus::Read),
            3 => Ok(NotifyStatus::Delete),
            _ => Err(KgsStatus::InvalidArgument),
        }
    }
}

impl NotifyStatus {
    pub fn to_id(&self) -> i32 {
        self.clone() as i32
    }

    pub fn get_comment(&self) -> String {
        match self {
            Self::Unread => "未讀",
            Self::Read => "已讀",
            Self::Delete => "刪除",
        }
        .to_string()
    }
}
