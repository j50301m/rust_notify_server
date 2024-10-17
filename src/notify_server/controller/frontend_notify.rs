use crate::notify_server::model;
use kgs_tracing::tracing;
use once_cell::sync::Lazy;
use protos::frontend_notify::{self, frontend_notify_service_server::*, ForwardNotifyRequest};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc::Sender, RwLock};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response};

pub static FRONTEND_NOTIFY_SERVER: Lazy<Arc<FrontendNotifyServer>> =
    Lazy::new(|| FrontendNotifyServer::default().into());
#[derive(Default, Debug)]
pub struct FrontendNotifyServer {
    pub connections:
        Arc<RwLock<HashMap<model::User, Sender<Result<frontend_notify::Receiver, tonic::Status>>>>>,
}

#[async_trait::async_trait]
impl FrontendNotifyService for FrontendNotifyServer {
    type CreateConnectionStream =
        tokio_stream::wrappers::ReceiverStream<Result<frontend_notify::Receiver, tonic::Status>>;

    #[tracing::instrument]
    async fn create_connection(
        &self,
        request: Request<frontend_notify::ConnectionRequest>,
    ) -> Result<Response<Self::CreateConnectionStream>, tonic::Status> {
        let receiver = self
            .create_connection(request.into_inner())
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(ReceiverStream::new(receiver)))
    }

    #[tracing::instrument]
    async fn close_connection(
        &self,
        request: Request<frontend_notify::ConnectionRequest>,
    ) -> Result<Response<frontend_notify::Empty>, tonic::Status> {
        self.close_connection(request.into_inner())
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(frontend_notify::Empty {}))
    }

    #[tracing::instrument]
    async fn system_to_frontend_user(
        &self,
        request: Request<frontend_notify::SendRequest>,
    ) -> Result<Response<frontend_notify::Empty>, tonic::Status> {
        self.system_to_frontend_user(request.into_inner())
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(frontend_notify::Empty {}))
    }

    #[tracing::instrument]
    async fn get_notify_records(
        &self,
        request: Request<frontend_notify::GetNotifyRecordRequest>,
    ) -> Result<Response<frontend_notify::GetNotifyRecordResponse>, tonic::Status> {
        let res = FrontendNotifyServer::get_notify_records(request.into_inner())
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(res))
    }

    #[tracing::instrument]
    async fn update_notify_records(
        &self,
        request: Request<frontend_notify::UpdateNotifyRecordRequest>,
    ) -> Result<Response<frontend_notify::UpdateNotifyRecordResponse>, tonic::Status> {
        let res = FrontendNotifyServer::update_notify_records(request.into_inner())
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(res))
    }

    #[tracing::instrument]
    async fn get_unread_notify_count(
        &self,
        request: Request<frontend_notify::GetUnreadNotifyCountRequest>,
    ) -> Result<Response<frontend_notify::GetUnreadNotifyCountResponse>, tonic::Status> {
        let res = FrontendNotifyServer::get_unread_notify_count(request.into_inner())
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(res))
    }

    #[tracing::instrument]
    async fn all_read(
        &self,
        request: Request<frontend_notify::AllReadRequest>,
    ) -> Result<Response<frontend_notify::Empty>, tonic::Status> {
        let res = FrontendNotifyServer::all_read(request.into_inner())
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(res))
    }

    #[tracing::instrument]
    async fn get_notify_by_id(
        &self,
        request: Request<frontend_notify::GetNotifyByIdRequest>,
    ) -> Result<Response<frontend_notify::Notify>, tonic::Status> {
        let res = FrontendNotifyServer::get_notify_by_id(request.into_inner())
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(res))
    }

    #[tracing::instrument]
    async fn forward_notify(
        &self,
        request: Request<ForwardNotifyRequest>,
    ) -> Result<Response<frontend_notify::Empty>, tonic::Status> {
        // parse the request
        let request = request.into_inner();
        let res = self
            .handle_forward_notify(request)
            .await
            .map_err(|err| err.to_tonic_status())?;

        Ok(Response::new(res))
    }
}
