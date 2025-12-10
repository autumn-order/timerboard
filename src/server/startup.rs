use crate::server::{config::Config, error::AppError};

/// Connects to the Sqlite database and runs pending migrations.
///
/// Establishes a connection pool to the Sqlite database using the connection string from
/// configuration, then automatically runs all pending SeaORM migrations to ensure the database
/// schema is up-to-date. This function must complete successfully before the application can
/// access the database.
///
/// # Arguments
/// - `config` - Application configuration containing the database URL
///
/// # Returns
/// - `Ok(DatabaseConnection)` - Connected database with migrations applied
/// - `Err(Error)` - Failed to connect to database or run migrations
pub async fn connect_to_database(config: &Config) -> Result<sea_orm::DatabaseConnection, AppError> {
    use migration::{Migrator, MigratorTrait};
    use sea_orm::{ConnectOptions, Database};

    let mut opt = ConnectOptions::new(&config.database_url);
    opt.sqlx_logging(false);

    let db = Database::connect(opt).await?;

    Migrator::up(&db, None).await?;

    Ok(db)
}
