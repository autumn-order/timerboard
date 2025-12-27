use std::ops::{Deref, DerefMut};

use dioxus::prelude::*;

/// Represents an item that needs to be fetched from the API
#[derive(Clone, Copy)]
pub struct Cache<T> {
    inner: Signal<CacheData<T>>,
}

impl<T: 'static> Cache<T> {
    pub fn new() -> Self {
        Self {
            inner: Signal::new(CacheData::default()),
        }
    }

    pub fn read(&self) -> impl Deref<Target = CacheData<T>> + '_ {
        self.inner.read()
    }

    pub fn write(&mut self) -> impl DerefMut<Target = CacheData<T>> + '_ {
        self.inner.write()
    }
}

impl<T: 'static> Default for Cache<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal cache data structure
#[derive(Clone)]
pub struct CacheData<T> {
    /// The cache data
    pub data: Option<T>,
    /// Bool indicating whether or not the data has been fetched
    pub fetched: bool,
}

impl<T> CacheData<T> {
    /// Updates the cache from a Result, marking it as fetched
    pub fn update_from_result<E>(&mut self, result: Result<T, E>) -> Result<(), E> {
        self.fetched = true;
        match result {
            Ok(data) => {
                self.data = Some(data);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Updates the cache from a Result<Option<T>>, marking it as fetched
    pub fn update_from_optional_result<E>(
        &mut self,
        result: Result<Option<T>, E>,
    ) -> Result<(), E> {
        self.fetched = true;
        match result {
            Ok(data) => {
                self.data = data;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

impl<T> Default for CacheData<T> {
    fn default() -> Self {
        Self {
            data: None,
            fetched: false,
        }
    }
}
