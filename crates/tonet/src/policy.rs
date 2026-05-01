//! Size and ingestion policy helpers.

use crate::limits::EngineLimits;

/// Returns `Ok(())` if `byte_len` is within the document budget.
#[inline]
pub fn check_document_size(byte_len: usize, limits: &EngineLimits) -> Result<(), DocumentTooLarge> {
    if byte_len > limits.max_document_bytes {
        Err(DocumentTooLarge {
            len: byte_len,
            max: limits.max_document_bytes,
        })
    } else {
        Ok(())
    }
}

/// Document exceeded [`EngineLimits::max_document_bytes`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DocumentTooLarge {
    pub len: usize,
    pub max: usize,
}
