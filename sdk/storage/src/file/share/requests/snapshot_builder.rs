use crate::file::prelude::*;
use azure_core::headers::{add_optional_header, add_optional_header_ref};
use azure_core::prelude::*;
use http::method::Method;
use http::status::StatusCode;
use std::convert::TryInto;

#[derive(Debug, Clone)]
pub struct SnapshotBuilder<'a> {
    share_client: &'a ShareClient,
    metadata: Option<&'a Metadata>,
    client_request_id: Option<ClientRequestId<'a>>,
    timeout: Option<Timeout>,
}

impl<'a> SnapshotBuilder<'a> {
    pub(crate) fn new(share_client: &'a ShareClient) -> Self {
        Self {
            share_client,
            metadata: None,
            client_request_id: None,
            timeout: None,
        }
    }

    setters! {
        metadata: &'a Metadata => Some(metadata),
        client_request_id: ClientRequestId<'a> => Some(client_request_id),
        timeout: Timeout => Some(timeout),
    }

    pub async fn execute(self) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let mut url = self.share_client.url_with_segments(None)?;

        url.query_pairs_mut().append_pair("restype", "share");
        url.query_pairs_mut().append_pair("comp", "snapshot");

        self.timeout.append_to_url_query(&mut url);

        let request = self.share_client.prepare_request(
            url.as_str(),
            &Method::GET,
            &|mut request| {
                request = add_optional_header(&self.metadata, request);
                request = add_optional_header(&self.client_request_id, request);
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

        Ok(())
    }
}
