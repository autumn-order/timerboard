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
    Fetched(T),
    Error(ApiError),
}

impl<T> CacheState<T> {
    pub fn is_fetched(&self) -> bool {
        !matches!(self, CacheState::NotFetched)
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
