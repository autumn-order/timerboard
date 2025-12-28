use std::{
    future::Future,
    ops::{Deref, DerefMut},
};

use dioxus::prelude::*;

use crate::client::model::error::ApiError;

/// Represents an item that needs to be fetched from the API
#[derive(Clone, Copy)]
pub struct Cache<T> {
    inner: Signal<CacheState<T>>,
}

impl<T: 'static> Cache<T> {
    pub fn new() -> Self {
        Self {
            inner: Signal::new(CacheState::default()),
        }
    }

    pub fn read(&self) -> impl Deref<Target = CacheState<T>> + '_ {
        self.inner.read()
    }

    pub fn write(&mut self) -> impl DerefMut<Target = CacheState<T>> + '_ {
        self.inner.write()
    }

    pub fn fetch<F, Fut>(&mut self, fetcher: F)
    where
        F: Fn() -> Fut + 'static,
        Fut: Future<Output = Result<T, ApiError>> + 'static,
        T: Clone,
    {
        self.fetch_internal(fetcher, true);
    }

    /// Refetches data even if already fetched, but skips if currently loading
    pub fn refetch<F, Fut>(&mut self, fetcher: F)
    where
        F: Fn() -> Fut + 'static,
        Fut: Future<Output = Result<T, ApiError>> + 'static,
        T: Clone,
    {
        self.fetch_internal(fetcher, false);
    }

    fn fetch_internal<F, Fut>(&mut self, fetcher: F, skip_if_fetched: bool)
    where
        F: Fn() -> Fut + 'static,
        Fut: Future<Output = Result<T, ApiError>> + 'static,
        T: Clone,
    {
        // Atomic check-and-set under a single write lock
        {
            let mut state = self.inner.write();
            if state.is_loading() || (skip_if_fetched && state.is_fetched()) {
                return;
            }
            *state = CacheState::Loading;
        }

        let future = use_resource(fetcher);

        if let Some(result) = &*future.read_unchecked() {
            let mut cache = self.inner.write();
            *cache = match result {
                Ok(data) => CacheState::Fetched(data.clone()),
                Err(e) => CacheState::Error(e.clone()),
            };
        }
    }
}

#[derive(Clone, Default)]
pub enum CacheState<T> {
    #[default]
    NotFetched,
    Loading,
    Fetched(T),
    Error(ApiError),
}

impl<T> CacheState<T> {
    pub fn is_fetched(&self) -> bool {
        !matches!(self, CacheState::NotFetched)
    }

    pub fn is_loading(&self) -> bool {
        matches!(self, CacheState::Loading)
    }

    pub fn data(&self) -> Option<&T> {
        match self {
            CacheState::Fetched(data) => Some(data),
            _ => None,
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, CacheState::Fetched(_))
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
