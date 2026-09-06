//! Helpers for safe validation binding diagnostics.

use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
pub(crate) fn path_error(kind: BindErrorKind) -> BindError {
    BindError::new(kind)
}
