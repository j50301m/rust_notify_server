use crate::{enums, notify_server::model::User};
use kgs_tracing::tracing;
use once_cell::sync::Lazy;
use protos::backstage_notify::{self, back_stage_notify_service_server::*};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response};

pub static BACKSTAGE_NOTIFY_SERVER: Lazy<Arc<BackstageNotifyServer>> =
    Lazy::new(|| BackstageNotifyServer::default().into());
#[derive(Debug)]
pub struct StreamInfo {
    pub role_ids: Vec<i64>,
    pub user_account: String,
    pub tx: tokio::sync::mpsc::Sender<Result<backstage_notify::Receiver, tonic::Status>>,
}

#[derive(Default, Debug)]
pub struct BackstageNotifyServer {
    pub connections: Arc<RwLock<HashMap<User, StreamInfo>>>,
}

#[async_trait::async_trait]
impl BackStageNotifyService for BackstageNotifyServer {
    type CreateConnectionStream =
        tokio_stream::wrappers::ReceiverStream<Result<backstage_notify::Receiver, tonic::Status>>;

    #[tracing::instrument]
    async fn create_connection(
        &self,
        request: Request<backstage_notify::ConnectionRequest>,
    ) -> Result<Response<Self::CreateConnectionStream>, tonic::Status> {
        let receiver = self
            .create_connection(request.into_inner())
            .await
            .map_err(|e| e.to_tonic_status())?;

        Ok(Response::new(ReceiverStream::new(receiver)))
    }

    #[tracing::instrument]
    async fn close_connection(
        &self,
        request: Request<backstage_notify::CloseRequest>,
    ) -> Result<Response<backstage_notify::Empty>, tonic::Status> {
        self.close_connection(request.into_inner())
            .await
            .map_err(|e| e.to_tonic_status())?;

        Ok(Response::new(backstage_notify::Empty {}))
    }

    #[tracing::instrument]
    async fn system_to_backstage_user(
        &self,
        request: Request<backstage_notify::SendRequest>,
    ) -> Result<Response<backstage_notify::Empty>, tonic::Status> {
        self.system_broadcast_to_backstage_user(request.into_inner())
            .await
            .map_err(|e| e.to_tonic_status())?;

        Ok(Response::new(backstage_notify::Empty {}))
    }

    #[tracing::instrument]
    async fn get_notify_records(
        &self,
        request: Request<backstage_notify::GetNotifyRecordRequest>,
    ) -> Result<Response<backstage_notify::GetNotifyRecordResponse>, tonic::Status> {
        let res = BackstageNotifyServer::get_notify_records(request.into_inner())
            .await
            .map_err(|e| e.to_tonic_status())?;

        Ok(Response::new(res))
    }

    #[tracing::instrument]
    async fn update_notify_records(
        &self,
        request: Request<backstage_notify::UpdateNotifyRecordRequest>,
    ) -> Result<Response<backstage_notify::UpdateNotifyRecordResponse>, tonic::Status> {
        let res = BackstageNotifyServer::update_notify_records(request.into_inner())
            .await
            .map_err(|e| e.to_tonic_status())?;

        Ok(Response::new(res))
    }

    #[tracing::instrument]
    async fn get_unread_notify_count(
        &self,
        request: Request<backstage_notify::GetUnreadNotifyCountRequest>,
    ) -> Result<Response<backstage_notify::GetUnreadNotifyCountResponse>, tonic::Status> {
        let res = BackstageNotifyServer::get_unread_notify_count(request.into_inner())
            .await
            .map_err(|e| e.to_tonic_status())?;

        Ok(Response::new(res))
    }

    #[tracing::instrument]
    async fn all_read(
        &self,
        request: Request<backstage_notify::AllReadRequest>,
    ) -> Result<Response<backstage_notify::Empty>, tonic::Status> {
        let res = BackstageNotifyServer::all_read(request.into_inner())
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(res))
    }

    #[tracing::instrument]
    async fn get_user_notify_records(
        &self,
        request: Request<backstage_notify::GetUserNotifyRecordRequest>,
    ) -> Result<Response<backstage_notify::GetUserNotifyRecordResponse>, tonic::Status> {
        let res = BackstageNotifyServer::get_user_notify_records(request.into_inner())
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(res))
    }

    #[tracing::instrument]
    async fn get_notify_by_id(
        &self,
        request: Request<backstage_notify::GetNotifyByIdRequest>,
    ) -> Result<Response<backstage_notify::Notify>, tonic::Status> {
        let res = BackstageNotifyServer::get_notify_by_id(request.into_inner())
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(res))
    }

    #[tracing::instrument]
    async fn backstage_send_to_user(
        &self,
        request: Request<backstage_notify::BackstageSendToUserRequest>,
    ) -> Result<Response<backstage_notify::Empty>, tonic::Status> {
        let res = BackstageNotifyServer::backstage_send_to_user(request.into_inner())
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(res))
    }

    #[tracing::instrument]
    async fn get_client_event_summary(
        &self,
        request: Request<backstage_notify::GetClientEventSummaryRequest>,
    ) -> Result<Response<backstage_notify::ClientEventSummaryList>, tonic::Status> {
        let res = BackstageNotifyServer::get_client_event_summary(request.into_inner())
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(res))
    }

    #[tracing::instrument]
    async fn get_client_templates(
        &self,
        request: Request<backstage_notify::GetClientTemplatesRequest>,
    ) -> Result<Response<backstage_notify::ClientTemplateList>, tonic::Status> {
        let res = BackstageNotifyServer::get_client_templates(request.into_inner())
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(res))
    }

    #[tracing::instrument]
    async fn get_notify_task_list(
        &self,
        request: Request<backstage_notify::GetNotifyTaskListRequest>,
    ) -> Result<Response<backstage_notify::NotifyTaskList>, tonic::Status> {
        let res = BackstageNotifyServer::get_notify_task_list(request.into_inner())
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(res))
    }

    #[tracing::instrument]
    async fn get_notify_task_details(
        &self,
        request: Request<backstage_notify::GetNotifyTaskDetailsRequest>,
    ) -> Result<Response<backstage_notify::NotifyTaskDetailList>, tonic::Status> {
        let res = BackstageNotifyServer::get_notify_task_details(request.into_inner())
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(res))
    }

    #[tracing::instrument]
    async fn get_client_event(
        &self,
        request: Request<backstage_notify::GetClientEventRequest>,
    ) -> Result<Response<backstage_notify::ClientEventList>, tonic::Status> {
        let res = BackstageNotifyServer::get_client_event(request.into_inner())
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(res))
    }

    #[tracing::instrument]
    async fn update_client_event(
        &self,
        request: Request<backstage_notify::UpdateClientEventRequest>,
    ) -> Result<Response<backstage_notify::Empty>, tonic::Status> {
        let res = BackstageNotifyServer::update_client_event(request.into_inner())
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(res))
    }

    #[tracing::instrument]
    async fn delete_client_event(
        &self,
        request: Request<backstage_notify::DeleteClientEventRequest>,
    ) -> Result<Response<backstage_notify::Empty>, tonic::Status> {
        let res = BackstageNotifyServer::delete_client_event(request.into_inner())
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(res))
    }

    #[tracing::instrument]
    async fn create_client_event(
        &self,
        request: Request<backstage_notify::CreateClientEventRequest>,
    ) -> Result<Response<backstage_notify::Empty>, tonic::Status> {
        let res = BackstageNotifyServer::create_client_event(request.into_inner())
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(res))
    }

    #[tracing::instrument]
    async fn forward_notify(
        &self,
        request: Request<backstage_notify::ForwardNotifyRequest>,
    ) -> Result<Response<backstage_notify::Empty>, tonic::Status> {
        let request = request.into_inner();

        let _res = self
            .handle_notify(
                request.client_id,
                &request.role_ids,
                enums::NotifyType::InApp,
                request.client_notify_event_id,
                &request.title,
                &request.content,
            )
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(backstage_notify::Empty {}))
    }
}
