use crate::share::Share;
use azure_core::incompletevector::IncompleteVector;
use azure_core::RequestId;

#[derive(Debug, Clone)]
pub struct ListSharesResponse {
    pub incomplete_vector: IncompleteVector<Share>,
    pub request_id: RequestId,
}

impl ListSharesResponse {
    pub fn is_complete(&self) -> bool {
        self.incomplete_vector.is_complete()
    }
}
