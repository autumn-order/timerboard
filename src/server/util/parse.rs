use crate::server::error::{internal::InternalError, AppError};

/// Parses a u64 value from String
///
/// # Arguments
/// - `value` - The String to attempt to parse into `u64`
///
/// # Returns
/// - `Ok(u64)` - Successfully parsed String to `u64`
/// - `Err(AppError::InternalError(ParseStringId))` - Failed to parse
///   the string as a u64
pub fn parse_u64_from_string(value: String) -> Result<u64, AppError> {
    let result = value
        .parse::<u64>()
        .map_err(|e| InternalError::ParseStringId {
            value: value,
            source: e,
        })?;

    Ok(result)
}
