use kgs_err::models::status::Status as KgsStatus;
use sea_orm::{entity::prelude::*, strum::Display};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Display)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum TaskStatus {
    #[strum(to_string = "Pending")]
    Pending = 1, // 待處理
    #[strum(to_string = "Success")]
    Success = 2, // 成功
    #[strum(to_string = "Fail")]
    Fail = 3, // 失敗
}
impl From<TaskStatus> for i32 {
    fn from(task_status: TaskStatus) -> Self {
        task_status as i32
    }
}

impl TryFrom<i32> for TaskStatus {
    type Error = KgsStatus;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(TaskStatus::Pending),
            2 => Ok(TaskStatus::Success),
            3 => Ok(TaskStatus::Fail),
            _ => Err(KgsStatus::InvalidArgument),
        }
    }
}

impl TaskStatus {
    pub fn to_id(&self) -> i32 {
        self.clone().into()
    }

    pub fn get_comment(&self) -> String {
        match self {
            TaskStatus::Pending => "待處理",
            TaskStatus::Success => "成功",
            TaskStatus::Fail => "失敗",
        }
        .to_string()
    }
}
