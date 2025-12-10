use oauth2::{CsrfToken, Scope};
use url::Url;

use crate::server::service::oauth::DiscordAuthService;

impl DiscordAuthService {
    pub fn login_url(&self) -> (Url, CsrfToken) {
        let (authorize_url, csrf_state) = self
            .oauth_client
            .authorize_url(|| CsrfToken::new_random())
            // Request scope to retrieve user information without email
            .add_scope(Scope::new("identify".to_string()))
            .url();

        (authorize_url, csrf_state)
    }
}
