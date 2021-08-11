use crate::file::prelude::*;
use crate::share::access_tier::AccessTier;
use azure_core::{
    headers::add_optional_header,
    prelude::*,
};
use http::{method::Method, status::StatusCode};

#[derive(Debug, Clone)]
pub struct CreateBuilder<'a> {
    share_client: &'a ShareClient,
    access_tier: Option<AccessTier>,
    metadata: Option<&'a Metadata>,
    client_request_id: Option<ClientRequestId<'a>>,
    timeout: Option<Timeout>,
}

impl<'a> CreateBuilder<'a> {
    pub(crate) fn new(share_client: &'a ShareClient) -> Self {
        Self {
            share_client,
            access_tier: None,
            metadata: None,
            client_request_id: None,
            timeout: None,
        }
    }

    setters! {
        access_tier: AccessTier => Some(access_tier),
        metadata: &'a Metadata => Some(metadata),
        client_request_id: ClientRequestId<'a> => Some(client_request_id),
        timeout: Timeout => Some(timeout),
    }

    pub async fn execute(self) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let mut url = self.share_client.url_with_segments(None)?;

        url.query_pairs_mut().append_pair("restype", "share");

        self.timeout.append_to_url_query(&mut url);

        let request = self.share_client.prepare_request(
            url.as_str(),
            &Method::PUT,
            &|mut request| {
                request = add_optional_header(&self.access_tier, request);
                request = add_optional_header(&self.metadata, request);
                request = add_optional_header(&self.client_request_id, request);
                request
            },
            None,
        )?;

        let _response = self
            .share_client
            .storage_client()
            .storage_account_client()
            .http_client()
            .execute_request_check_status(request.0, StatusCode::CREATED)
            .await?;

        // TODO: Capture and return the response headers
        Ok(())
    }
}
