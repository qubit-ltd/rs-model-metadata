//! Errors raised by the model validation execution boundary.

use qubit_validator::next::{ExecutionError, ValidationReport};

/// An infrastructure error together with the report collected before it.
pub struct ModelValidationError {
    error: ExecutionError,
    partial_report: ValidationReport,
}

impl ModelValidationError {
    pub(crate) fn new(error: ExecutionError, partial_report: ValidationReport) -> Self {
        Self { error, partial_report }
    }
    /// Returns the execution failure.
    #[must_use]
    pub const fn error(&self) -> &ExecutionError { &self.error }
    /// Returns the report collected before the failure.
    #[must_use]
    pub const fn partial_report(&self) -> &ValidationReport { &self.partial_report }
    /// Consumes the error and returns its parts.
    #[must_use]
    pub fn into_parts(self) -> (ExecutionError, ValidationReport) { (self.error, self.partial_report) }
}

impl std::fmt::Debug for ModelValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("ModelValidationError").field("error", &self.error).field("partial_report", &self.partial_report).finish()
    }
}

impl std::fmt::Display for ModelValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.error.fmt(formatter) }
}

impl std::error::Error for ModelValidationError {}
