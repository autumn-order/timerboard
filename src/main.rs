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
            bot, config::Config, scheduler::fleet_notifications,
            service::admin::code::AdminCodeService, startup, state::AppState,
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

        // Initialize Discord bot and extract HTTP client
        let bot_db = db.clone();
        let (bot_client, discord_http) = bot::start::init_bot(&config, bot_db).await?;

        // Start Discord bot in a separate task
        tokio::spawn(async move {
            if let Err(e) = bot::start::start_bot(bot_client).await {
                tracing::error!("Discord bot error: {}", e);
            }
        });

        // Check for admin users and generate login link if none exist
        startup::check_for_admin(&db, &config, &admin_code_service).await?;

        // Start fleet notification scheduler
        let scheduler_db = db.clone();
        let scheduler_http = discord_http.clone();
        let scheduler_app_url = config.app_url.clone();
        tokio::spawn(async move {
            if let Err(e) = fleet_notifications::start_scheduler(
                scheduler_db,
                scheduler_http,
                scheduler_app_url,
            )
            .await
            {
                tracing::error!("Fleet notification scheduler error: {}", e);
            }
        });

        let mut router = dioxus::server::router(App);
        let server_routes = server::router::router()
            .with_state(AppState::new(
                db,
                http_client,
                oauth_client,
                admin_code_service,
                discord_http,
                config.app_url.clone(),
            ))
            .layer(session);
        router = router.merge(server_routes);

        Ok(router)
    })
}
