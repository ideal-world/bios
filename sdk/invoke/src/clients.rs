use std::fmt::Display;

use serde::Serialize;
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    web::{
        poem_openapi::types::{ParseFromJSON, ToJSON},
        web_client::TardisHttpResponse,
        web_resp::TardisResp,
    },
    TardisFuns, TardisFunsInst,
};

use crate::invoke_constants::TARDIS_CONTEXT;

#[cfg(feature = "spi_base")]
mod base_spi_client;
#[cfg(feature = "spi_kv")]
pub mod spi_kv_client;
#[cfg(feature = "spi_log")]
pub mod spi_log_client;
#[cfg(feature = "spi_search")]
pub mod spi_search_client;

#[cfg(feature = "iam")]
pub mod iam_client;
#[macro_export]
///
///
/// # Usage
/// as if `Client` is some type implemented trait `SimpleInvokeClient`
/// ```no_run, ignore
/// # pub struct Client<'a> {
/// #     base_url: &'a str,
/// #     ctx: &'a TardisContext,
/// #     funs: &'a TardisFunsInst,
/// # }
/// #
/// # impl<'a> Client<'a> {
/// #     pub fn new(base_url: &'a str, ctx: &'a TardisContext, funs: &'a TardisFunsInst) -> Self {
/// #         Self { base_url, funs, ctx }
/// #     }
/// # }
/// #
/// # impl SimpleInvokeClient for Client<'_> {
/// #     const DOMAIN_CODE: &'static str = "crate::consts::DOMAIN_CODE";
/// #
/// #     fn get_ctx(&self) -> &tardis::basic::dto::TardisContext {
/// #         self.ctx
/// #     }
/// #
/// #     fn get_base_url(&self) -> &str {
/// #         self.base_url
/// #     }
/// # }
///
/// impl_taidis_api_client!{
///     Client<'_>:
///     // <function_name>  <method>    <path>                                  <query> <response type>
///     { my_get_method,    get         ["/path", patharg, "path2", patharg2]   {code}  String}
///     //                                   query can be optional, if type not specified, it will be `&str`
///     { paginate_msg_log, post ["/ct/msg"] {page_number?: u32, page_size?} ReachMessageAddReq => String }
///     // query can omitted
///     { add_message, post ["/ct/msg"] ReachMessageAddReq => String }
///     { delete_test, delete ["/delete/some/msg"] String }
/// }
/// ```
macro_rules! impl_taidis_api_client {
    ($Client:ty: $({$($defs: tt)*})*) => {
        impl $Client {
            $(
                $crate::taidis_api!{$($defs)*}
            )*
        }
    }
}

#[macro_export]
///
///
/// ```ignore,no_run
/// taidis_api!{
///     mail_pwd_send, put ["/cc/msg/mail", mail] {message, subject} () => ()
/// }
/// ```
macro_rules! taidis_api {
    /*
       enter
     */
    ($fn_name:ident, $($tt:tt)*) => {
        $crate::taidis_api!($fn_name @method $($tt)*);
    };
    /*
        method
     */
    ($fn_name:ident @method $method:ident $($tt:tt)*) => {
        $crate::taidis_api!($fn_name @path $method {} {} $($tt)*);
    };
    /*
        path
     */
    ($fn_name:ident @path $method:ident {$($args_i:ident:$args_t:ty,)*} {$($path:expr,)*} [] $($tt:tt)*) => {
        $crate::taidis_api!($fn_name @query $method {$($args_i:$args_t,)*} {$($path,)*} {;} $($tt)*);
    };
    ($fn_name:ident @path $method:ident {$($args_i:ident:$args_t:ty,)*} {$($path:expr,)*} [$next:ident, $($rest_path:tt)*] $($tt:tt)*) => {
        $crate::taidis_api!($fn_name @path $method {$($args_i:$args_t,)* $next: &str,} {$($path,)* $next,} [$($rest_path)*] $($tt)*);
    };
    ($fn_name:ident @path $method:ident {$($args_i:ident:$args_t:ty,)*} {$($path:expr,)*} [$next:ident $(,)?] $($tt:tt)*) => {
        $crate::taidis_api!($fn_name @path $method {$($args_i:$args_t,)* $next: &str,} {$($path,)* $next,} [] $($tt)*);
    };
    ($fn_name:ident @path $method:ident {$($args_i:ident:$args_t:ty,)*} {$($path:expr,)*} [$next:literal, $($rest_path:tt)*] $($tt:tt)*) => {
        $crate::taidis_api!($fn_name @path $method {$($args_i:$args_t,)*} {$($path,)* $next,} [$($rest_path)*] $($tt)*);
    };
    ($fn_name:ident @path $method:ident {$($args_i:ident:$args_t:ty,)*} {$($path:expr,)*} [$next:literal $(,)?] $($tt:tt)*) => {
        $crate::taidis_api!($fn_name @path $method {$($args_i:$args_t,)*} {$($path,)* $next,} [] $($tt)*);
    };
    ($fn_name:ident @path $method:ident {$($args_i:ident:$args_t:ty,)*} {$($path:expr,)*} [$next_ident:ident:$next_type:ty, $($rest_path:tt)*] $($tt:tt)*) => {
        $crate::taidis_api!($fn_name @path $method {$($args_i:$args_t,)* $next_ident:$next_type,} {$($path,)* $next_ident.to_string().as_str(),} [$($rest_path)*] $($tt)*);
    };
    ($fn_name:ident @path $method:ident {$($args_i:ident:$args_t:ty,)*} {$($path:expr,)*} [$next_ident:ident:$next_type:ty $(,)?] $($tt:tt)*) => {
        $crate::taidis_api!($fn_name @path $method {$($args_i:$args_t,)* $next_ident:$next_type,} {$($path,)* $next_ident.to_string().as_str(),} [] $($tt)*);
    };
    /*
        query
        fn_name @query method {args} {paths} {query1, query2, ; optional_query1, optional_query2} {rest_querys} tt
     */
    ($fn_name:ident @query $method:ident {$($args_i:ident:$args_t:ty,)*} {$($path:expr,)*} {$($query:expr,)*;$($optional_query:expr,)*} {} $($tt:tt)*) => {
        $crate::taidis_api!($fn_name @build $method {$($args_i:$args_t,)*} {$($path,)*} {$($query,)*;$($optional_query,)*} $($tt)*);
    };
    // for ident:type
    ($fn_name:ident @query $method:ident {$($args_i:ident:$args_t:ty,)*} {$($path:expr,)*} {$($query:expr,)*;$($optional_query:expr,)*} {$next_ident:ident:$next_type:ty, $($rest_query:tt)*} $($tt:tt)*) => {
        $crate::taidis_api!($fn_name @query $method {$($args_i:$args_t,)* $next_ident:$next_type,} {$($path,)*} {$($query,)* (stringify!($next_ident), $next_ident), ; $($optional_query,)*} {$($rest_query)*} $($tt)*);
    };
    ($fn_name:ident @query $method:ident {$($args_i:ident:$args_t:ty,)*} {$($path:expr,)*} {$($query:expr,)*;$($optional_query:expr,)*} {$next_ident:ident:$next_type:ty $(,)?} $($tt:tt)*) => {
        $crate::taidis_api!($fn_name @query $method {$($args_i:$args_t,)* $next_ident:$next_type,} {$($path,)*} {$($query,)* (stringify!($next_ident), $next_ident), ; $($optional_query,)*} {} $($tt)*);
    };
    // for ident?:type
    ($fn_name:ident @query $method:ident {$($args_i:ident:$args_t:ty,)*} {$($path:expr,)*} {$($query:expr,)*;$($optional_query:expr,)*} {$next_ident:ident?:$next_type:ty, $($rest_query:tt)*} $($tt:tt)*) => {
        $crate::taidis_api!($fn_name @query $method {$($args_i:$args_t,)* $next_ident:Option<$next_type>,} {$($path,)*} {$($query,)* ; $($optional_query,)* (stringify!($next_ident),  $next_ident), } {$($rest_query)*} $($tt)*);
    };
    ($fn_name:ident @query $method:ident {$($args_i:ident:$args_t:ty,)*} {$($path:expr,)*} {$($query:expr,)*;$($optional_query:expr,)*} {$next_ident:ident?:$next_type:ty $(,)?} $($tt:tt)*) => {
        $crate::taidis_api!($fn_name @query $method {$($args_i:$args_t,)* $next_ident:Option<$next_type>,} {$($path,)*} {$($query,)* ; $($optional_query,)* (stringify!($next_ident),  $next_ident), } {} $($tt)*);
    };
    // for ident
    ($fn_name:ident @query $method:ident {$($args_i:ident:$args_t:ty,)*} {$($path:expr,)*} {$($query:expr,)*;$($optional_query:expr,)*} {$next:ident $(,)?} $($tt:tt)*) => {
        $crate::taidis_api!($fn_name @query $method {$($args_i:$args_t,)* $next:&str, } {$($path,)*} {$($query,)* (stringify!($next), $next), ; $($optional_query,)*} {} $($tt)*);
    };
    ($fn_name:ident @query $method:ident {$($args_i:ident:$args_t:ty,)*} {$($path:expr,)*} {$($query:expr,)*;$($optional_query:expr,)*} {$next:ident, $($rest_query:tt)*} $($tt:tt)*) => {
        $crate::taidis_api!($fn_name @query $method {$($args_i:$args_t,)* $next:&str, } {$($path,)*} {$($query,)* (stringify!($next), $next), ; $($optional_query,)*} {$($rest_query)*} $($tt)*);
    };
    // for ident?
    ($fn_name:ident @query $method:ident {$($args_i:ident:$args_t:ty,)*} {$($path:expr,)*} {$($query:expr,)*;$($optional_query:expr,)*} {$next:ident? $(,)?} $($tt:tt)*) => {
        $crate::taidis_api!($fn_name @query $method {$($args_i:$args_t,)* $next:Option<&str>, } {$($path,)*} {$($query,)* ; $($optional_query,)* (stringify!($next), $next), } {} $($tt)*);
    };
    ($fn_name:ident @query $method:ident {$($args_i:ident:$args_t:ty,)*} {$($path:expr,)*} {$($query:expr,)*;$($optional_query:expr,)*} {$next:ident?, $($rest_query:tt)*} $($tt:tt)*) => {
        $crate::taidis_api!($fn_name @query $method {$($args_i:$args_t,)* $next:Option<&str>, } {$($path,)*} {$($query,)* ; $($optional_query,)* (stringify!($next), $next), } {$($rest_query)*} $($tt)*);
    };
    // no query args
    ($fn_name:ident @query $method:ident {$($args_i:ident:$args_t:ty,)*} {$($path:expr,)*} {;} $($tt:tt)*) => {
        $crate::taidis_api!($fn_name @build $method {$($args_i:$args_t,)*} {$($path,)*} {;} $($tt)*);
    };
    /*
        build
     */
    ($fn_name:ident @build get {$($args_i:ident:$args_t:ty,)*} {$($path:expr,)*} {$($query:expr,)*;$($optional_query:expr,)*} $Resp:ty) => {
        pub async fn $fn_name(&self, $($args_i:$args_t,)*) -> tardis::basic::result::TardisResult<$Resp> {
            use $crate::clients::SimpleInvokeClient;
            use tardis::web::web_resp::TardisResp;
            let mut query = $crate::clients::QueryBuilder::new();
            $(
                {
                    let q = $query;
                    query.add(q.0, q.1);
                }
            )*
            $(
                {
                    let q = $optional_query;
                    query.add_optional(q.0, q.1);
                }
            )*
            let url = self.get_url(&[$($path,)*], query.as_ref());
            let header = self.get_tardis_context_header()?;
            let resp = self.get_funs().web_client().get::<TardisResp<$Resp>>(&url, Some(vec![header])).await?;
            Self::extract_response(resp)
        }
    };
    ($fn_name:ident @build post {$($args_i:ident:$args_t:ty,)*} {$($path:expr,)*} {$($query:expr,)*;$($optional_query:expr,)*} $Body:ty => $Resp:ty) => {
        pub async fn $fn_name(&self, $($args_i:$args_t,)* body: &$Body) -> tardis::basic::result::TardisResult<$Resp> {
            use $crate::clients::SimpleInvokeClient;
            use tardis::web::web_resp::TardisResp;
            let mut query = $crate::clients::QueryBuilder::new();
            $(
                {
                    let q = $query;
                    query.add(q.0, q.1);
                }
            )*
            $(
                {
                    let q = $optional_query;
                    query.add_optional(q.0, q.1);
                }
            )*
            let url = self.get_url(&[$($path,)*], query.as_ref());
            let header = self.get_tardis_context_header()?;
            let resp = self.get_funs().web_client().post::<$Body, TardisResp<$Resp>>(&url, body, Some(vec![header])).await?;
            Self::extract_response(resp)
        }
    };
    ($fn_name:ident @build put {$($args_i:ident:$args_t:ty,)*} {$($path:expr,)*} {$($query:expr,)*;$($optional_query:expr,)*} $Body:ty => $Resp:ty) => {
        pub async fn $fn_name(&self, $($args_i:$args_t,)* body: &$Body) -> tardis::basic::result::TardisResult<$Resp> {
            use $crate::clients::SimpleInvokeClient;
            use tardis::web::web_resp::TardisResp;
            let mut query = $crate::clients::QueryBuilder::new();
            $(
                {
                    let q = $query;
                    query.add(q.0, q.1);
                }
            )*
            $(
                {
                    let q = $optional_query;
                    query.add_optional(q.0, q.1);
                }
            )*
            let url = self.get_url(&[$($path,)*], query.as_ref());
            let header = self.get_tardis_context_header()?;
            let resp = self.get_funs().web_client().put::<$Body, TardisResp<$Resp>>(&url, body, Some(vec![header])).await?;
            Self::extract_response(resp)
        }
    };
    ($fn_name:ident @build delete {$($args_i:ident:$args_t:ty,)*} {$($path:expr,)*} {$($query:expr,)*;$($optional_query:expr,)*} $Resp:ty) => {
        pub async fn $fn_name(&self, $($args_i:$args_t),*) -> tardis::basic::result::TardisResult<$Resp> {
            use $crate::clients::SimpleInvokeClient;
            use tardis::web::web_resp::TardisResp;
            let mut query = $crate::clients::QueryBuilder::new();
            $(
                {
                    let q = $query;
                    query.add(q.0, q.1);
                }
            )*
            $(
                {
                    let q = $optional_query;
                    query.add_optional(q.0, q.1);
                }
            )*
            let url = self.get_url(&[$($path,)*], query.as_ref());
            let header = self.get_tardis_context_header()?;
            let resp = self.get_funs().web_client().delete::<TardisResp<$Resp>>(&url, Some(vec![header])).await?;
            Self::extract_response(resp)
        }
    };
}

#[derive(Debug, Default)]
pub struct QueryBuilder {
    pub inner: String,
}
impl AsRef<str> for QueryBuilder {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}
impl QueryBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add<T: Display>(&mut self, key: &'static str, val: T) {
        if !self.inner.is_empty() {
            self.inner.push('&')
        }
        self.inner.push_str(key);
        self.inner.push('=');
        self.inner.push_str(&val.to_string());
    }
    pub fn add_optional<T: Display>(&mut self, key: &'static str, val: Option<T>) {
        if let Some(val) = val {
            self.add(key, val)
        }
    }
}
pub trait SimpleInvokeClient {
    const DOMAIN_CODE: &'static str;
    fn get_ctx(&self) -> &TardisContext;
    fn get_funs(&self) -> &TardisFunsInst;
    fn get_base_url(&self) -> &str;

    /*
     * default implements
     */
    fn get_tardis_context_header(&self) -> TardisResult<(String, String)> {
        let ctx = self.get_ctx();
        Ok((TARDIS_CONTEXT.to_string(), TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(ctx)?)))
    }
    fn get_url(&self, path: &[&str], query: &str) -> String {
        format!(
            "{base}/{path}{path_query_spliter}{query}",
            // domain = Self::DOMAIN_CODE,
            base = self.get_base_url().trim_end_matches('/'),
            path = path.join("/").trim_matches('/'),
            path_query_spliter = if query.is_empty() { "" } else { "?" },
            query = query
        )
    }
    fn extract_response<T>(resp: TardisHttpResponse<TardisResp<T>>) -> TardisResult<T>
    where
        T: ParseFromJSON + ToJSON + Serialize + Send + Sync,
    {
        resp.body.map_or_else(
            || {
                Err(TardisError::internal_error(
                    &format!("invoke {domain} encounter an error", domain = Self::DOMAIN_CODE),
                    "500-invoke-request-error",
                ))
            },
            |b| {
                b.data.ok_or_else(|| TardisError {
                    code: b.code,
                    message: format!(
                        "simple invoke client call domain [{domain}] encounter an error: {msg}",
                        domain = Self::DOMAIN_CODE,
                        msg = b.msg
                    ),
                })
            },
        )
    }
}
