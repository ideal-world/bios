use http::HeaderName;
use spacegate_shell::{
    hyper::Request,
    kernel::{extension::MatchedSgRouter, service::http_route::match_request::HttpPathMatchRewrite},
    SgBody,
};

pub mod op_redis_header_expand;
pub mod op_redis_status;
pub mod op_redis_allow_api;

fn redis_format_key(req: &Request<SgBody>, matched: &MatchedSgRouter, header: &str) -> Option<String> {
    let is_method_any_match = matched.method.as_ref().is_none();
    let method = if !is_method_any_match { req.method().as_str() } else { "*" };
    let path = matched
        .path
        .as_ref()
        .map(|p| match p {
            HttpPathMatchRewrite::Exact(path, _) => path,
            HttpPathMatchRewrite::Prefix(path, _) => path,
            HttpPathMatchRewrite::RegExp(regex, _) => regex.as_str(),
        })
        .unwrap_or("*");
    let header = req.headers().get(header).and_then(|v| v.to_str().ok())?;
    Some(format!("{}:{}:{}", method, path, header))
}
