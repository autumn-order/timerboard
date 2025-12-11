use rand::Rng;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Time-to-live for admin codes in seconds
const ADMIN_CODE_TTL_SECONDS: u64 = 60;

/// Stored admin code with expiration timestamp
#[derive(Clone)]
struct AdminCode {
    code: String,
    expires_at: Instant,
}

impl AdminCode {
    fn new(code: String) -> Self {
        Self {
            code,
            expires_at: Instant::now() + Duration::from_secs(ADMIN_CODE_TTL_SECONDS),
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }

    fn matches(&self, input: &str) -> bool {
        self.code == input
    }
}

/// Service for managing temporary admin codes used for initial admin user setup.
///
/// The admin code is generated once on server startup if no admin user exists,
/// and is stored in memory with a 60-second TTL. It can be validated once and
/// then is automatically invalidated after successful use or expiration.
#[derive(Clone)]
pub struct AdminCodeService {
    code: Arc<RwLock<Option<AdminCode>>>,
}

impl AdminCodeService {
    /// Creates a new AdminCodeService instance.
    pub fn new() -> Self {
        Self {
            code: Arc::new(RwLock::new(None)),
        }
    }

    /// Generates a new random admin code and stores it with a 60-second TTL.
    ///
    /// The code is a cryptographically secure random string of 32 characters
    /// using alphanumeric characters.
    ///
    /// # Returns
    /// The generated admin code string.
    pub async fn generate(&self) -> String {
        let code_string = Self::generate_random_code();
        let admin_code = AdminCode::new(code_string.clone());
        *self.code.write().await = Some(admin_code);
        code_string
    }

    /// Validates the provided code against the stored admin code.
    ///
    /// If validation is successful, the code is automatically invalidated
    /// to prevent reuse. Expired codes are also invalidated and fail validation.
    ///
    /// # Arguments
    /// * `input_code` - The code to validate
    ///
    /// # Returns
    /// `true` if the code matches and was valid (not expired), `false` otherwise.
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
    /// # Returns
    /// A 32-character random string.
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
    /// This method also cleans up expired codes.
    ///
    /// # Returns
    /// `true` if a valid, non-expired code is stored, `false` otherwise.
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

    #[tokio::test]
    async fn test_generate_code() {
        let service = AdminCodeService::new();
        assert!(!service.has_valid_code().await);

        let code = service.generate().await;
        assert_eq!(code.len(), 32);
        assert!(service.has_valid_code().await);
    }

    #[tokio::test]
    async fn test_validate_correct_code() {
        let service = AdminCodeService::new();
        let code = service.generate().await;

        assert!(service.validate_and_consume(&code).await);
        // Code should be consumed after validation
        assert!(!service.has_valid_code().await);
    }

    #[tokio::test]
    async fn test_validate_incorrect_code() {
        let service = AdminCodeService::new();
        service.generate().await;

        assert!(!service.validate_and_consume("wrong_code").await);
        // Code should still exist after failed validation
        assert!(service.has_valid_code().await);
    }

    #[tokio::test]
    async fn test_validate_without_code() {
        let service = AdminCodeService::new();
        assert!(!service.validate_and_consume("any_code").await);
    }

    #[tokio::test]
    async fn test_invalidate_code() {
        let service = AdminCodeService::new();
        service.generate().await;
        assert!(service.has_valid_code().await);

        service.invalidate().await;
        assert!(!service.has_valid_code().await);
    }

    #[tokio::test]
    async fn test_code_cannot_be_reused() {
        let service = AdminCodeService::new();
        let code = service.generate().await;

        assert!(service.validate_and_consume(&code).await);
        // Trying to use the same code again should fail
        assert!(!service.validate_and_consume(&code).await);
    }

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
