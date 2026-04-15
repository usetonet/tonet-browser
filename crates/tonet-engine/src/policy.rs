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
    use super::*;
    use crate::EngineLimits;

    #[test]
    fn document_within_budget_ok() {
        assert!(check_document_size(100, &EngineLimits::STANDARD).is_ok());
    }

    #[test]
    fn document_over_budget_err() {
        let lim = EngineLimits {
            max_document_bytes: 10,
            ..EngineLimits::STANDARD
        };
        let e = check_document_size(11, &lim).unwrap_err();
        assert_eq!(e.max, 10);
        assert_eq!(e.len, 11);
    }
}
