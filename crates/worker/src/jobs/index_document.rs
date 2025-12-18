//! Index document job for full document indexing pipeline.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Job to fully index a document (chunk, embed, store)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDocumentJob {
    pub job_id: Uuid,
    pub document_id: Uuid,
}

impl IndexDocumentJob {
    pub fn new(document_id: Uuid) -> Self {
        Self {
            job_id: Uuid::new_v4(),
            document_id,
        }
    }
}
