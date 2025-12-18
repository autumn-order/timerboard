//! Admin code service for managing temporary verification codes.
//!
//! This module provides the `AdminCodeService` for generating and validating one-time-use
//! admin verification codes. These codes are used during initial application setup to create
//! the first admin user. Codes are stored in-memory with a 60-second TTL and are automatically
//! invalidated after successful use or expiration.

use rand::Rng;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Time-to-live for admin codes in seconds.
const ADMIN_CODE_TTL_SECONDS: u64 = 60;

/// Stored admin code with expiration timestamp.
///
/// Represents a temporary admin verification code that expires after a fixed duration.
/// Used internally by AdminCodeService to track code validity and expiration state.
#[derive(Clone)]
struct AdminCode {
    /// The verification code string.
    code: String,
    /// Timestamp when this code expires.
    expires_at: Instant,
}

impl AdminCode {
    /// Creates a new admin code with 60-second TTL.
    ///
    /// # Arguments
    /// - `code` - The verification code string
    ///
    /// # Returns
    /// - `AdminCode` - New admin code instance that expires in 60 seconds
    fn new(code: String) -> Self {
        Self {
            code,
            expires_at: Instant::now() + Duration::from_secs(ADMIN_CODE_TTL_SECONDS),
        }
    }

    /// Checks if the admin code has expired.
    ///
    /// Compares the current time against the expiration timestamp.
    ///
    /// # Returns
    /// - `true` - Code has expired
    /// - `false` - Code is still valid
    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }

    /// Checks if the provided input matches this admin code.
    ///
    /// Performs an exact string comparison between the stored code and input.
    ///
    /// # Arguments
    /// - `input` - The code string to compare against
    ///
    /// # Returns
    /// - `true` - Input matches the stored code
    /// - `false` - Input does not match
    fn matches(&self, input: &str) -> bool {
        self.code == input
    }
}

/// Service for managing temporary admin codes used for initial admin user setup.
///
/// Provides methods for generating one-time-use verification codes that allow the first
/// user to authenticate with admin privileges during application setup. The admin code is
/// generated once on server startup if no admin user exists, stored in memory with a
/// 60-second TTL, and automatically invalidated after successful use or expiration.
/// This ensures secure initial setup without requiring pre-configured credentials.
#[derive(Clone)]
pub struct AdminCodeService {
    /// The currently active admin code, if any.
    code: Arc<RwLock<Option<AdminCode>>>,
}

impl AdminCodeService {
    /// Creates a new AdminCodeService instance.
    ///
    /// Initializes the service with no active admin code. Codes must be explicitly
    /// generated via the `generate` method.
    ///
    /// # Returns
    /// - `AdminCodeService` - New service instance with no active code
    pub fn new() -> Self {
        Self {
            code: Arc::new(RwLock::new(None)),
        }
    }

    /// Generates a new random admin code and stores it with a 60-second TTL.
    ///
    /// Creates a cryptographically secure random 32-character alphanumeric string
    /// and stores it in memory. Any previously generated code is replaced. The code
    /// can be validated once using `validate_and_consume` and expires after 60 seconds.
    /// Used during server startup when no admin user exists.
    ///
    /// # Returns
    /// - `String` - The generated 32-character admin verification code
    pub async fn generate(&self) -> String {
        let code_string = Self::generate_random_code();
        let admin_code = AdminCode::new(code_string.clone());
        *self.code.write().await = Some(admin_code);
        code_string
    }

    /// Validates the provided code against the stored admin code.
    ///
    /// Checks if the input code matches the stored admin code and has not expired.
    /// If validation is successful, the code is automatically invalidated to prevent
    /// reuse (one-time-use). Expired codes are also invalidated and fail validation.
    /// Used during OAuth callback to verify the user should receive admin privileges.
    ///
    /// # Arguments
    /// - `input_code` - The code string to validate
    ///
    /// # Returns
    /// - `true` - Code matches and was valid (not expired), code has been consumed
    /// - `false` - Code doesn't match, is expired, or no code exists
    pub async fn validate_and_consume(&self, input_code: &str) -> bool {
        let mut code = self.code.write().await;

        if let Some(stored_code) = code.as_ref() {
            // Check if code is expired
            if stored_code.is_expired() {
                // Invalidate expired code
                *code = None;
                return false;
            }

            // Check if code matches
            if stored_code.matches(input_code) {
                // Invalidate the code after successful validation
                *code = None;
                return true;
            }
        }

        false
    }

    /// Generates a cryptographically secure random alphanumeric code.
    ///
    /// Creates a 32-character string using uppercase letters, lowercase letters,
    /// and digits (0-9). Uses the system's random number generator for security.
    ///
    /// # Returns
    /// - `String` - A 32-character random alphanumeric string
    fn generate_random_code() -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                 abcdefghijklmnopqrstuvwxyz\
                                 0123456789";
        const CODE_LENGTH: usize = 32;

        let mut rng = rand::rng();

        (0..CODE_LENGTH)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Checks if an admin code currently exists and is valid (not expired).
    ///
    /// Verifies that a code is stored and has not expired. Automatically cleans up
    /// expired codes by invalidating them. Used in tests to verify code state.
    ///
    /// # Returns
    /// - `true` - A valid, non-expired code is stored
    /// - `false` - No code exists or the stored code has expired
    #[cfg(test)]
    pub async fn has_valid_code(&self) -> bool {
        let mut code = self.code.write().await;

        if let Some(stored_code) = code.as_ref() {
            if stored_code.is_expired() {
                // Clean up expired code
                *code = None;
                return false;
            }
            return true;
        }

        false
    }

    /// Invalidates the current admin code if one exists.
    ///
    /// Removes the stored admin code, preventing it from being validated.
    /// Used in tests to reset the service state between test cases.
    #[cfg(test)]
    pub async fn invalidate(&self) {
        *self.code.write().await = None;
    }
}

impl Default for AdminCodeService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    /// Tests generating a new admin code.
    ///
    /// Verifies that generating a code creates a 32-character string and stores
    /// it as a valid code in the service.
    ///
    /// Expected: Ok with 32-character code and valid code state
    #[tokio::test]
    async fn test_generate_code() {
        let service = AdminCodeService::new();
        assert!(!service.has_valid_code().await);

        let code = service.generate().await;
        assert_eq!(code.len(), 32);
        assert!(service.has_valid_code().await);
    }

    /// Tests validating a correct admin code.
    ///
    /// Verifies that validating with the correct code returns true and automatically
    /// consumes the code, preventing reuse.
    ///
    /// Expected: Ok with successful validation and consumed code
    #[tokio::test]
    async fn test_validate_correct_code() {
        let service = AdminCodeService::new();
        let code = service.generate().await;

        assert!(service.validate_and_consume(&code).await);
        // Code should be consumed after validation
        assert!(!service.has_valid_code().await);
    }

    /// Tests validating an incorrect admin code.
    ///
    /// Verifies that validating with an incorrect code returns false and preserves
    /// the stored code for future validation attempts.
    ///
    /// Expected: Ok with failed validation and code still valid
    #[tokio::test]
    async fn test_validate_incorrect_code() {
        let service = AdminCodeService::new();
        service.generate().await;

        assert!(!service.validate_and_consume("wrong_code").await);
        // Code should still exist after failed validation
        assert!(service.has_valid_code().await);
    }

    /// Tests validating when no code exists.
    ///
    /// Verifies that validation fails gracefully when no code has been generated.
    ///
    /// Expected: Ok with failed validation
    #[tokio::test]
    async fn test_validate_without_code() {
        let service = AdminCodeService::new();
        assert!(!service.validate_and_consume("any_code").await);
    }

    /// Tests manual code invalidation.
    ///
    /// Verifies that calling invalidate removes the stored code.
    ///
    /// Expected: Ok with code removed
    #[tokio::test]
    async fn test_invalidate_code() {
        let service = AdminCodeService::new();
        service.generate().await;
        assert!(service.has_valid_code().await);

        service.invalidate().await;
        assert!(!service.has_valid_code().await);
    }

    /// Tests that admin codes cannot be reused.
    ///
    /// Verifies that after successful validation, the same code cannot be used again.
    ///
    /// Expected: Ok with first validation succeeding and second failing
    #[tokio::test]
    async fn test_code_cannot_be_reused() {
        let service = AdminCodeService::new();
        let code = service.generate().await;

        assert!(service.validate_and_consume(&code).await);
        // Trying to use the same code again should fail
        assert!(!service.validate_and_consume(&code).await);
    }

    /// Tests that admin codes expire after TTL.
    ///
    /// Verifies that codes become invalid after 60 seconds and are automatically
    /// cleaned up when checked or validated.
    ///
    /// Expected: Ok with code valid initially and expired after TTL
    #[tokio::test]
    async fn test_code_expires_after_ttl() {
        let service = AdminCodeService::new();
        let code = service.generate().await;

        // Code should be valid initially
        assert!(service.has_valid_code().await);

        // Wait for code to expire (61 seconds)
        sleep(Duration::from_secs(ADMIN_CODE_TTL_SECONDS + 1)).await;

        // Code should be expired and automatically invalidated
        assert!(!service.has_valid_code().await);
        assert!(!service.validate_and_consume(&code).await);
    }

    /// Tests that expired code validation fails.
    ///
    /// Verifies that attempting to validate an expired code returns false.
    ///
    /// Expected: Ok with validation failing for expired code
    #[tokio::test]
    async fn test_expired_code_validation_fails() {
        let service = AdminCodeService::new();
        let code = service.generate().await;

        // Wait for code to expire
        sleep(Duration::from_secs(ADMIN_CODE_TTL_SECONDS + 1)).await;

        // Expired code should fail validation
        assert!(!service.validate_and_consume(&code).await);
    }
}
