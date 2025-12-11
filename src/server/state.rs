use oauth2::basic::{BasicErrorResponseType, BasicTokenType};
use oauth2::{
    Client, EmptyExtraTokenFields, EndpointNotSet, EndpointSet, RevocationErrorResponseType,
    StandardErrorResponse, StandardRevocableToken, StandardTokenIntrospectionResponse,
    StandardTokenResponse,
};
use sea_orm::DatabaseConnection;

use super::service::admin::AdminCodeService;

pub(crate) type OAuth2Client = Client<
    StandardErrorResponse<BasicErrorResponseType>,
    StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
>;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub http_client: reqwest::Client,
    pub oauth_client: OAuth2Client,
    pub admin_code_service: AdminCodeService,
}

impl AppState {
    pub fn new(
        db: DatabaseConnection,
        http_client: reqwest::Client,
        oauth_client: OAuth2Client,
        admin_code_service: AdminCodeService,
    ) -> Self {
        Self {
            db,
            http_client,
            oauth_client,
            admin_code_service,
        }
    }
}
