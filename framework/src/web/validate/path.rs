use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

use actix_router::PathDeserializer;
use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest};
use futures::future::{ready, Ready};
use serde::de::{Deserialize, DeserializeOwned};
use validator::Validate;

use crate::web::validate::error::{DeserializeErrors, Error};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Path<T> {
    inner: T,
}

#[deprecated(note = "Please, use actix_web_validator::Path instead.", since = "2.0.0")]
pub type ValidatedPath<T> = Path<T>;

impl<T> Path<T> {
    /// Deconstruct to an inner value
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> AsRef<T> for Path<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> Deref for Path<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T: fmt::Debug> fmt::Debug for Path<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T: fmt::Display> fmt::Display for Path<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> FromRequest for Path<T>
where
    T: DeserializeOwned + Validate,
{
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = PathConfig;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let error_handler = req.app_data::<Self::Config>().map(|c| c.ehandler.clone()).unwrap_or(None);
        ready(
            Deserialize::deserialize(PathDeserializer::new(req.match_info()))
                .map_err(|error| Error::Deserialize(DeserializeErrors::DeserializePath(error)))
                .and_then(|value: T| value.validate().map(move |_| value).map_err(Error::Validate))
                .map(|inner| Path { inner })
                .map_err(move |e| {
                    log::debug!(
                        "Failed during Path extractor deserialization. \
                         Request path: {:?}",
                        req.path()
                    );
                    if let Some(error_handler) = error_handler {
                        (error_handler)(e, req)
                    } else {
                        actix_web::error::ErrorNotFound(e)
                    }
                }),
        )
    }
}

#[derive(Clone)]
pub struct PathConfig {
    ehandler: Option<Arc<dyn Fn(Error, &HttpRequest) -> actix_web::Error + Send + Sync>>,
}

impl PathConfig {
    /// Set custom error handler
    pub fn error_handler<F>(mut self, f: F) -> Self
    where
        F: Fn(Error, &HttpRequest) -> actix_web::Error + Send + Sync + 'static,
    {
        self.ehandler = Some(Arc::new(f));
        self
    }
}

impl Default for PathConfig {
    fn default() -> Self {
        Self { ehandler: None }
    }
}
