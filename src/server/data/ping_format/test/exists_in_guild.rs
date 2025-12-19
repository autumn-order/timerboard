use super::*;

/// Tests checking if ping format exists in guild.
///
/// Verifies that the repository correctly returns true when a ping format
/// exists and belongs to the specified guild.
///
/// Expected: Ok(true)
#[tokio::test]
async fn returns_true_for_existing_format_in_guild() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;
    let ping_format = factory::create_ping_format(db, &guild.guild_id).await?;

    let repo = PingFormatRepository::new(db);
    let result = repo
        .exists_in_guild(ping_format.id, guild.guild_id.parse().unwrap())
        .await;

    assert!(result.is_ok());
    assert!(result.unwrap());

    Ok(())
}

/// Tests checking if ping format exists in different guild.
///
/// Verifies that the repository correctly returns false when a ping format
/// exists but belongs to a different guild.
///
/// Expected: Ok(false)
#[tokio::test]
async fn returns_false_for_format_in_different_guild() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild1 = factory::create_guild(db).await?;
    let guild2 = factory::create_guild(db).await?;
    let ping_format = factory::create_ping_format(db, &guild1.guild_id).await?;

    let repo = PingFormatRepository::new(db);
    let result = repo
        .exists_in_guild(ping_format.id, guild2.guild_id.parse().unwrap())
        .await;

    assert!(result.is_ok());
    assert!(!result.unwrap());

    Ok(())
}

/// Tests checking if nonexistent ping format exists.
///
/// Verifies that the repository correctly returns false when checking
/// for a ping format ID that doesn't exist in the database.
///
/// Expected: Ok(false)
#[tokio::test]
async fn returns_false_for_nonexistent_format() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;

    let repo = PingFormatRepository::new(db);
    let result = repo
        .exists_in_guild(999999, guild.guild_id.parse().unwrap())
        .await;

    assert!(result.is_ok());
    assert!(!result.unwrap());

    Ok(())
}

/// Tests checking if format exists in nonexistent guild.
///
/// Verifies that the repository correctly returns false when checking
/// for a format in a guild that doesn't exist.
///
/// Expected: Ok(false)
#[tokio::test]
async fn returns_false_for_nonexistent_guild() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;
    let ping_format = factory::create_ping_format(db, &guild.guild_id).await?;

    let repo = PingFormatRepository::new(db);
    let result = repo.exists_in_guild(ping_format.id, 999999999).await;

    assert!(result.is_ok());
    assert!(!result.unwrap());

    Ok(())
}

/// Tests checking if deleted format exists.
///
/// Verifies that the repository correctly returns false after a ping format
/// has been deleted.
///
/// Expected: Ok(false)
#[tokio::test]
async fn returns_false_for_deleted_format() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;
    let ping_format = factory::create_ping_format(db, &guild.guild_id).await?;
    let format_id = ping_format.id;
    let guild_id = guild.guild_id.parse().unwrap();

    // Delete the format
    entity::prelude::PingFormat::delete_by_id(format_id)
        .exec(db)
        .await?;

    let repo = PingFormatRepository::new(db);
    let result = repo.exists_in_guild(format_id, guild_id).await;

    assert!(result.is_ok());
    assert!(!result.unwrap());

    Ok(())
}

/// Tests checking multiple formats in same guild.
///
/// Verifies that the repository correctly identifies which formats belong
/// to a specific guild when multiple formats exist.
///
/// Expected: Ok(true) for formats in guild, Ok(false) for others
#[tokio::test]
async fn checks_multiple_formats_in_same_guild() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild = factory::create_guild(db).await?;
    let guild_id = guild.guild_id.parse().unwrap();

    let format1 = factory::create_ping_format(db, &guild.guild_id).await?;
    let format2 = factory::create_ping_format(db, &guild.guild_id).await?;
    let format3 = factory::create_ping_format(db, &guild.guild_id).await?;

    let repo = PingFormatRepository::new(db);

    let result1 = repo.exists_in_guild(format1.id, guild_id).await?;
    let result2 = repo.exists_in_guild(format2.id, guild_id).await?;
    let result3 = repo.exists_in_guild(format3.id, guild_id).await?;

    assert!(result1);
    assert!(result2);
    assert!(result3);

    Ok(())
}

/// Tests checking formats across multiple guilds.
///
/// Verifies that the repository correctly validates format ownership
/// when multiple guilds each have their own formats.
///
/// Expected: Ok(true) only for correct guild-format pairs
#[tokio::test]
async fn checks_formats_across_multiple_guilds() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild1 = factory::create_guild(db).await?;
    let guild2 = factory::create_guild(db).await?;
    let guild3 = factory::create_guild(db).await?;

    let format1 = factory::create_ping_format(db, &guild1.guild_id).await?;
    let format2 = factory::create_ping_format(db, &guild2.guild_id).await?;
    let format3 = factory::create_ping_format(db, &guild3.guild_id).await?;

    let repo = PingFormatRepository::new(db);

    // Check correct associations
    assert!(
        repo.exists_in_guild(format1.id, guild1.guild_id.parse().unwrap())
            .await?
    );
    assert!(
        repo.exists_in_guild(format2.id, guild2.guild_id.parse().unwrap())
            .await?
    );
    assert!(
        repo.exists_in_guild(format3.id, guild3.guild_id.parse().unwrap())
            .await?
    );

    // Check incorrect associations
    assert!(
        !repo
            .exists_in_guild(format1.id, guild2.guild_id.parse().unwrap())
            .await?
    );
    assert!(
        !repo
            .exists_in_guild(format2.id, guild3.guild_id.parse().unwrap())
            .await?
    );
    assert!(
        !repo
            .exists_in_guild(format3.id, guild1.guild_id.parse().unwrap())
            .await?
    );

    Ok(())
}

/// Tests authorization pattern for format access.
///
/// Verifies that the exists_in_guild method can be used for authorization
/// checks before allowing operations on a ping format.
///
/// Expected: Ok(true) for authorized access, Ok(false) for unauthorized
#[tokio::test]
async fn validates_authorization_pattern() -> Result<(), DbErr> {
    let test = TestBuilder::new()
        .with_table(DiscordGuild)
        .with_table(PingFormat)
        .build()
        .await
        .unwrap();
    let db = test.db.as_ref().unwrap();

    let guild1 = factory::create_guild(db).await?;
    let guild2 = factory::create_guild(db).await?;
    let ping_format = factory::create_ping_format(db, &guild1.guild_id).await?;

    let repo = PingFormatRepository::new(db);

    // Guild1 can access their format
    let authorized = repo
        .exists_in_guild(ping_format.id, guild1.guild_id.parse().unwrap())
        .await?;
    assert!(authorized);

    // Guild2 cannot access guild1's format
    let unauthorized = repo
        .exists_in_guild(ping_format.id, guild2.guild_id.parse().unwrap())
        .await?;
    assert!(!unauthorized);

    Ok(())
}
