mod certificate;
mod config;
mod user;

pub use certificate::*;
pub use config::*;
pub use user::*;

/// Implement this trait to validate the request.
///
/// # Example:
///
/// ```
/// use ssp_backend_types::ValidateRequest;
///
/// struct CreateCertificateRequest {
///     title: String,
///     content: Vec<u8>,
/// }
///
/// impl ValidateRequest for CreateCertificateRequest {
///     fn validate(&self) -> Result<(), String> {
///         if self.title.is_empty() {
///             return Err("Title cannot be empty.".to_string());
///         }
///
///         if self.title.chars().count() > 100 {
///             return Err(format!(
///                 "Title cannot be longer than {} characters.",
///                 100
///             ));
///         }
///
///         if self.content.is_empty() {
///             return Err("Content cannot be empty.".to_string());
///         }
///
///         Ok(())
///     }
/// }
/// ```
pub trait ValidateRequest {
    /// Validate the request, based on its fields.
    fn validate(&self) -> Result<(), String>;
}
