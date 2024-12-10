use warden_core::{
    google::protobuf::Empty,
    pseudonyms::{
        CreateAccountHolderRequest, DeleteAccountHolderRequest, GetAccountHolderRequest,
        GetAccountHolderResponse, UpdateAccountHolderRequest,
    },
};

pub mod server;

pub struct State {}

#[tonic::async_trait]
impl warden_core::pseudonyms::account_holder_service_server::AccountHolderService for State {
    #[must_use]
    async fn create_account_holder(
        &self,
        request: tonic::Request<CreateAccountHolderRequest>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        todo!()
    }

    #[must_use]
    async fn get_account_holder(
        &self,
        request: tonic::Request<GetAccountHolderRequest>,
    ) -> Result<tonic::Response<GetAccountHolderResponse>, tonic::Status> {
        todo!()
    }

    #[must_use]
    async fn update_account_holder(
        &self,
        request: tonic::Request<UpdateAccountHolderRequest>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        todo!()
    }

    #[must_use]
    async fn delete_account_holder(
        &self,
        request: tonic::Request<DeleteAccountHolderRequest>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        todo!()
    }
}
