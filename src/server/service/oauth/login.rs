use oauth2::CsrfToken;
use url::Url;

use crate::server::service::oauth::DiscordAuthService;

impl DiscordAuthService {
    pub fn login_url(&self) -> (Url, CsrfToken) {
        let (authorize_url, csrf_state) = self
            .oauth_client
            .authorize_url(|| CsrfToken::new_random())
            .url();

        (authorize_url, csrf_state)
    }
}
