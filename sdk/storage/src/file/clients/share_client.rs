use crate::core::clients::{StorageAccountClient, StorageClient};
use crate::share::requests::*;
use azure_core::prelude::*;
use bytes::Bytes;
use http::method::Method;
use http::request::{Builder, Request};
use std::sync::Arc;

pub trait AsShareClient<CN: Into<String>> {
    fn as_share_client(&self, share_name: CN) -> Arc<ShareClient>;
}

impl<CN: Into<String>> AsShareClient<CN> for Arc<StorageClient> {
    fn as_share_client(&self, share_name: CN) -> Arc<ShareClient> {
        ShareClient::new(self.clone(), share_name.into())
    }
}

#[derive(Debug, Clone)]
pub struct ShareClient {
    storage_client: Arc<StorageClient>,
    share_name: String,
}

impl ShareClient {
    pub(crate) fn new(storage_client: Arc<StorageClient>, share_name: String) -> Arc<Self> {
        Arc::new(Self {
            storage_client,
            share_name,
        })
    }

    pub fn share_name(&self) -> &str {
        &self.share_name
    }

    pub(crate) fn storage_client(&self) -> &StorageClient {
        self.storage_client.as_ref()
    }

    pub(crate) fn http_client(&self) -> &dyn HttpClient {
        self.storage_client.storage_account_client().http_client()
    }

    pub(crate) fn storage_account_client(&self) -> &StorageAccountClient {
        self.storage_client.storage_account_client()
    }

    pub(crate) fn url_with_segments<'a, I>(
        &'a self,
        segments: I,
    ) -> Result<url::Url, url::ParseError>
    where
        I: IntoIterator<Item = &'a str>,
    {
        self.storage_client
            .file_url_with_segments(Some(self.share_name.as_str()).into_iter().chain(segments))
    }

    pub fn create(&self) -> CreateBuilder {
        CreateBuilder::new(self)
    }

    pub fn delete(&self) -> DeleteBuilder {
        DeleteBuilder::new(self)
    }

    pub fn get_acl(&self) -> GetACLBuilder {
        GetACLBuilder::new(self)
    }

    pub fn get_properties(&self) -> GetPropertiesBuilder {
        GetPropertiesBuilder::new(self)
    }

    pub(crate) fn prepare_request(
        &self,
        url: &str,
        method: &Method,
        http_header_adder: &dyn Fn(Builder) -> Builder,
        request_body: Option<Bytes>,
    ) -> Result<(Request<Bytes>, url::Url), crate::Error> {
        self.storage_client
            .prepare_request(url, method, http_header_adder, request_body)
    }
}
