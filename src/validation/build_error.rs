//! Helpers for safe validation binding diagnostics.

use qubit_validator::next::BindError;
use qubit_validator::next::BindErrorKind;
pub(crate) fn path_error(kind: BindErrorKind) -> BindError {
    BindError::new(kind)
}
