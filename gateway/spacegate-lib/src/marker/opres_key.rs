use http::HeaderValue;
use spacegate_shell::{
    kernel::{extension::MatchedSgRouter, layers::http_route::match_request::SgHttpPathMatch, Marker},
    spacegate_ext_redis::{redis::ToRedisArgs, AsRedisKey},
};

const BIOS_AUTH_HEADER: &str = "Bios-Authorization";

pub struct OpresKey {
    pub method: String,
    pub path: String,
    pub ak: String,
}

impl OpresKey {
    pub fn new(method: String, path: String, ak: String) -> Self {
        Self { method, path, ak }
    }
}

impl Marker for OpresKey {
    fn extract(req: &http::Request<spacegate_shell::SgBody>) -> Option<Self> {
        let matched = req.extensions().get::<MatchedSgRouter>()?;
        let method = if matched.method.is_none() { "*" } else { req.method().as_str() };
        let path = match matched.path {
            Some(SgHttpPathMatch::Prefix(ref path)) => path.as_str(),
            _ => "*",
        };
        let header = req.headers().get(BIOS_AUTH_HEADER)?.clone();
        let (ak, _sign) = header.to_str().ok()?.split_once(':')?;
        Some(Self::new(method.to_string(), path.to_string(), ak.to_string()))
    }
}

impl AsRedisKey for OpresKey {
    fn as_redis_key(&self, prefix: impl AsRef<str>) -> String {
        format!("{}:{}:{}:{}", prefix.as_ref(), self.method, self.path, self.ak)
    }
}

impl ToRedisArgs for OpresKey {
    fn to_redis_args(&self) -> Vec<Vec<u8>> {
        vec![]
        // vec![self.method.as_str().as_bytes().to_vec(), self.path.as_bytes().to_vec()]
    }

    fn write_redis_args<W>(&self, _out: &mut W)
    where
        W: ?Sized + tardis::cache::RedisWrite,
    {
        // self.method.as_str().write_redis_args(out);
        // self.path.as_str().write_redis_args(out);
    }
}
