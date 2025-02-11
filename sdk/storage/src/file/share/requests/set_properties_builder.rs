use crate::file::prelude::*;
use crate::share::access_tier::AccessTier;
use crate::share::responses::GetPropertiesResponse;
use azure_core::headers::{add_optional_header, add_optional_header_ref};
use azure_core::prelude::*;
use http::method::Method;
use http::status::StatusCode;
use std::convert::TryInto;

#[derive(Debug, Clone)]
pub struct GetPropertiesBuilder<'a> {
    share_client: &'a ShareClient,
    quota: Option<u64>,
    access_tier: Option<AccessTier>,
    client_request_id: Option<ClientRequestId<'a>>,
    timeout: Option<Timeout>,
    lease_id: Option<&'a LeaseId>,
}

impl<'a> GetPropertiesBuilder<'a> {
    pub(crate) fn new(share_client: &'a ShareClient) -> Self {
        Self {
            share_client,
            quota: None,
            access_tier: None,
            client_request_id: None,
            timeout: None,
            lease_id: None,
        }
    }

    setters! {
        quota: u64 => Some(quota),
        access_tier: AccessTier: Some(AccessTier),
        client_request_id: ClientRequestId<'a> => Some(client_request_id),
        timeout: Timeout => Some(timeout),
        lease_id: &'a LeaseId => Some(lease_id),
    }

    pub async fn execute(
        &self,
    ) -> Result<GetPropertiesResponse, Box<dyn std::error::Error + Sync + Send>> {
        let mut url = self.share_client.url_with_segments(None)?;

        url.query_pairs_mut().append_pair("restype", "share");

        self.timeout.append_to_url_query(&mut url);

        let request = self.share_client.prepare_request(
            url.as_str(),
            &Method::HEAD,
            &|mut request| {
                request = add_optional_header(&self.client_request_id, request);
                request = add_optional_header_ref(&self.lease_id, request);
                request
            },
            None,
        )?;

        let response = self
            .share_client
            .storage_client()
            .storage_account_client()
            .http_client()
            .execute_request_check_status(request.0, StatusCode::OK)
            .await?;

        Ok((self.share_client.share_name(), response.headers()).try_into()?)
    }
}
