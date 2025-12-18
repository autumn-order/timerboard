//! Administrative services for bot setup and access control.
//!
//! This module provides administrative services for managing the application's Discord bot
//! integration and controlling admin access through temporary verification codes. It includes
//! services for generating bot invitation URLs and managing one-time-use admin codes during
//! initial application setup.
//!
//! ## Admin Code Flow
//!
//! When the application starts without any admin users:
//! 1. An admin code is generated and logged to the console
//! 2. The first user to log in with this code becomes an admin
//! 3. The code expires after 60 seconds or upon successful use
//! 4. Subsequent admin users must be granted privileges by existing admins
//!
//! ## Bot Invitation Flow
//!
//! Administrators can generate bot invitation URLs to add the bot to their Discord servers:
//! 1. Admin requests a bot invitation URL from the API
//! 2. Service generates OAuth2 URL with required scopes and permissions
//! 3. Admin follows the URL to Discord's authorization flow
//! 4. Discord redirects back to the callback URL after authorization
//! 5. Bot becomes available in the authorized guild

pub mod bot;
pub mod code;
