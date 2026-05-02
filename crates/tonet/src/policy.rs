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

#[cfg(test)]
mod tests {
    use super::{check_document_size, DocumentTooLarge};
    use crate::limits::EngineLimits;

    #[test]
    fn at_budget_ok_one_over_err() {
        let limits = EngineLimits {
            max_document_bytes: 100,
            ..EngineLimits::STANDARD
        };
        assert!(check_document_size(100, &limits).is_ok());
        assert_eq!(
            check_document_size(101, &limits),
            Err(DocumentTooLarge {
                len: 101,
                max: 100,
            })
        );
    }

    #[test]
    fn zero_byte_document_ok() {
        let limits = EngineLimits {
            max_document_bytes: 0,
            ..EngineLimits::STANDARD
        };
        assert!(check_document_size(0, &limits).is_ok());
        assert!(check_document_size(1, &limits).is_err());
    }
}
