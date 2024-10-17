use crate::enums;
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "backstage_send_task_detail")]

pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub backstage_send_task_id: i64,      // 任務id
    pub notify_level: enums::NotifyLevel, // 通知等級 1.一般 2.系統 3.重要
    pub notify_type: enums::NotifyType,   // 通知管道 1.站內信 2.信箱 3.簡訊
    pub title: String,                    // 發送當下的模板標題
    pub content: String,                  // 發送當下的模板內容
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn to_publish_model(self) -> crate::mq_manager::TemplateModel {
        crate::mq_manager::TemplateModel {
            notify_type: self.notify_type,
            title: self.title,
            content: self.content,
            notify_level: self.notify_level,
        }
    }
}

impl Model {
    pub fn to_proto<T: ProtoTrait>(self) -> T {
        T::to_proto(self)
    }
}

pub trait ProtoTrait {
    fn to_proto(model: Model) -> Self;
}

impl ProtoTrait for protos::backstage_notify::NotifyTaskDetail {
    fn to_proto(model: Model) -> Self {
        protos::backstage_notify::NotifyTaskDetail {
            notify_type: model.notify_type as i32,
            title: model.title,
            content: model.content,
        }
    }
}
