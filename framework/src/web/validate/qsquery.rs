use std::{fmt, ops};
use std::ops::Deref;
use std::sync::Arc;

use actix_web::{FromRequest, HttpRequest};
use futures::future::{err, ok, Ready};
use serde::de;
use serde_qs::Config as QsConfig;
use validator::Validate;

use crate::web::validate::error::Error;

pub struct QsQueryConfig {
    ehandler: Option<Arc<dyn Fn(Error, &HttpRequest) -> actix_web::Error + Send + Sync>>,
    qs_config: QsConfig,
}

impl QsQueryConfig {
    /// Set custom error handler
    pub fn error_handler<F>(mut self, f: F) -> Self
        where
            F: Fn(Error, &HttpRequest) -> actix_web::Error + Send + Sync + 'static,
    {
        self.ehandler = Some(Arc::new(f));
        self
    }

    /// Set custom serialization parameters
    pub fn qs_config(mut self, config: QsConfig) -> Self {
        self.qs_config = config;
        self
    }
}

impl Default for QsQueryConfig {
    fn default() -> Self {
        QsQueryConfig {
            ehandler: None,
            qs_config: QsConfig::default(),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct QsQuery<T>(pub T);

impl<T> AsRef<T> for QsQuery<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for QsQuery<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> ops::DerefMut for QsQuery<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: fmt::Debug> fmt::Debug for QsQuery<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Display> fmt::Display for QsQuery<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> QsQuery<T>
    where
        T: Validate,
{
    /// Deconstruct to an inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> FromRequest for QsQuery<T>
    where
        T: de::DeserializeOwned + Validate,
{
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = QsQueryConfig;

    /// Builds Query struct from request and provides validation mechanism
    #[inline]
    fn from_request(
        req: &actix_web::web::HttpRequest,
        _: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let query_config = req.app_data::<QsQueryConfig>();

        let error_handler = query_config.map(|c| c.ehandler.clone()).unwrap_or(None);

        let default_qsconfig = QsConfig::default();
        let qsconfig = query_config
            .map(|c| &c.qs_config)
            .unwrap_or(&default_qsconfig);

        qsconfig
            .deserialize_str::<T>(req.query_string())
            .map_err(Error::from)
            .and_then(|value| {
                value
                    .validate()
                    .map(move |_| value)
                    .map_err(Error::Validate)
            })
            .map_err(move |e| {
                log::debug!(
                    "Failed during Query extractor validation. \
                     Request path: {:?}",
                    req.path()
                );
                if let Some(error_handler) = error_handler {
                    (error_handler)(e, req)
                } else {
                    e.into()
                }
            })
            .map(|value| ok(QsQuery(value)))
            .unwrap_or_else(err)
    }
}
