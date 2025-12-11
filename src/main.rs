mod client;
mod model;

#[cfg(feature = "server")]
mod server;

use client::App;

fn main() {
    #[cfg(not(feature = "server"))]
    dioxus::launch(App);

    #[cfg(feature = "server")]
    dioxus::serve(|| async move {
        use dioxus_logger::tracing;

        use crate::server::{
            bot, config::Config, service::admin::code::AdminCodeService, startup, state::AppState,
        };

        dotenvy::dotenv().ok();
        let config = Config::from_env()?;

        let db = startup::connect_to_database(&config).await?;
        let session = startup::connect_to_session(&db).await?;
        let http_client = startup::setup_reqwest_client();
        let oauth_client = startup::setup_oauth_client(&config);

        // Create admin code service
        let admin_code_service = AdminCodeService::new();

        tracing::info!("Starting server");

        // Start Discord bot in a separate task
        let bot_db = db.clone();
        let bot_config = config.clone();
        tokio::spawn(async move {
            if let Err(e) = bot::start::start_bot(&bot_config, bot_db).await {
                tracing::error!("Discord bot error: {}", e);
            }
        });

        // Check for admin users and generate login link if none exist
        startup::check_for_admin(&db, &config, &admin_code_service).await?;

        let mut router = dioxus::server::router(App);
        let server_routes = server::router::router()
            .with_state(AppState::new(
                db,
                http_client,
                oauth_client,
                admin_code_service,
            ))
            .layer(session);
        router = router.merge(server_routes);

        Ok(router)
    })
}
