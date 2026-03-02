pub mod user_schemas;
pub mod auth_schemas;  // Add this line
pub mod password_reset_schemas;  // Add this line

pub use user_schemas::{CreateUserRequest, UpdateUserRequest, UserResponse};
pub use auth_schemas::*;  // Export all auth schemas
pub use password_reset_schemas::*;  // Add this line