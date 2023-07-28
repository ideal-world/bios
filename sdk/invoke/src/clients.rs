use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    web::web_client::TardisHttpResponse,
    TardisFuns,
};

use crate::invoke_constants::TARDIS_CONTEXT;

#[cfg(feature = "spi-base")]
mod base_spi_client;
#[cfg(feature = "iam")]
pub mod iam_client;
#[cfg(feature = "spi-kv")]
pub mod spi_kv_client;
#[cfg(feature = "spi-log")]
pub mod spi_log_client;

pub trait InvokeClient {
    const DOMAIN_CODE: &'static str;
    fn get_ctx(&self) -> &TardisContext;
    fn get_base_url(&self) -> &str;

    /*
     * default implements
     */
    fn get_tardis_context_header(&self) -> TardisResult<(String, String)> {
        let ctx = self.get_ctx();
        Ok((TARDIS_CONTEXT.to_string(), TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(ctx)?)))
    }
    fn get_url<'a>(&self, path: &[&str], query: impl IntoIterator<Item = (&'a str, &'a str)>) -> String {
        let mut query_part = String::new();
        for (k, v) in query {
            query_part.push(if query_part.is_empty() { '?' } else { '&' });
            query_part.push_str(k.as_ref());
            query_part.push('=');
            query_part.push_str(v.as_ref());
        }
        format!(
            "{base}/{domain}/{path}{query}",
            domain = Self::DOMAIN_CODE,
            base = self.get_base_url().trim_end_matches('/'),
            path = path.join("/").trim_matches('/'),
            query = query_part
        )
    }
    fn extract_response<T>(resp: TardisHttpResponse<T>) -> TardisResult<T> {
        resp.body.ok_or_else(|| {
            TardisError::internal_error(
                &format!("invoke-{domain}: {code}", domain = Self::DOMAIN_CODE, code = resp.code),
                "500-invoke-request-error",
            )
        })
    }
}
