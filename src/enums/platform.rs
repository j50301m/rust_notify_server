use kgs_err::models::status::Status as KgsStatus;
use sea_orm::{entity::prelude::*, strum::Display};
use serde::{Deserialize, Serialize};

/// 通知平台
#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Display, Deserialize, Serialize,
)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum Platform {
    #[strum(to_string = "Frontend")]
    Frontend = 1, // 前台
    #[strum(to_string = "Backstage")]
    Backstage = 2, // 後台
    #[strum(to_string = "MasterBackstage")]
    MasterBackstage = 3, // 總管理後台
}

impl From<Platform> for i32 {
    fn from(platform: Platform) -> Self {
        platform as i32
    }
}

impl TryFrom<i32> for Platform {
    type Error = KgsStatus;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Platform::Frontend),
            2 => Ok(Platform::Backstage),
            3 => Ok(Platform::MasterBackstage),
            _ => Err(KgsStatus::InvalidArgument),
        }
    }
}

impl Platform {
    pub fn to_id(&self) -> i32 {
        self.clone().into()
    }

    pub fn get_comment(&self) -> String {
        match self {
            Platform::Frontend => "前台",
            Platform::Backstage => "後台",
            Platform::MasterBackstage => "總管理後台",
        }
        .to_string()
    }
}
