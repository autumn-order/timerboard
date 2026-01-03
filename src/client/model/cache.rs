use crate::client::model::error::ApiError;

#[derive(Clone, Default)]
pub enum Cache<T> {
    #[default]
    NotFetched,
    Loading,
    Fetched(T),
    Error(ApiError),
}

impl<T> Cache<T> {
    pub fn is_fetched(&self) -> bool {
        !matches!(self, Cache::NotFetched)
    }

    pub fn is_loading(&self) -> bool {
        matches!(self, Cache::Loading)
    }

    pub fn data(&self) -> Option<&T> {
        match self {
            Cache::Fetched(data) => Some(data),
            _ => None,
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Cache::Fetched(_))
    }

    /// Map the inner data to another value, returning None if not fetched successfully
    pub fn map<U, F>(&self, f: F) -> Option<U>
    where
        F: FnOnce(&T) -> U,
    {
        self.data().map(f)
    }

    /// Flat-map the inner data, useful for chaining Options
    pub fn and_then<U, F>(&self, f: F) -> Option<U>
    where
        F: FnOnce(&T) -> Option<U>,
    {
        self.data().and_then(f)
    }
}

#[derive(Clone, Default)]
pub enum GuildCache<T> {
    #[default]
    NotFetched,
    Loading {
        guild_id: u64,
    },
    Fetched {
        guild_id: u64,
        data: T,
    },
    Error {
        guild_id: u64,
        error: ApiError,
    },
}

impl<T> GuildCache<T> {
    pub fn guild_id(&self) -> Option<u64> {
        match self {
            GuildCache::Loading { guild_id } | GuildCache::Fetched { guild_id, .. } => {
                Some(*guild_id)
            }
            GuildCache::Error { guild_id, .. } => Some(*guild_id),
            GuildCache::NotFetched => None,
        }
    }

    pub fn data(&self) -> Option<&T> {
        match self {
            GuildCache::Fetched { data, .. } => Some(data),
            _ => None,
        }
    }

    pub fn is_fetched(&self) -> bool {
        matches!(self, GuildCache::Fetched { .. })
    }

    pub fn is_loading(&self) -> bool {
        matches!(self, GuildCache::Loading { .. })
    }
}
