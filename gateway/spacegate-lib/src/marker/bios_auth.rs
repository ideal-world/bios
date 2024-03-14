use http::{HeaderValue, Method};
use spacegate_shell::{
    kernel::Marker,
    spacegate_ext_redis::{redis::ToRedisArgs, AsRedisKey},
};

const BIOS_AUTH_HEADER: &str = "Bios-Authorization";

pub struct BiosAuth {
    pub method: Method,
    pub path: String,
    pub ak: HeaderValue,
}

impl BiosAuth {
    pub fn new(method: Method, path: String, ak: HeaderValue) -> Self {
        Self { method, path, ak }
    }
}

impl Marker for BiosAuth {
    fn extract(req: &http::Request<spacegate_shell::SgBody>) -> Option<Self> {
        let method = req.method().clone();
        let path = req.uri().path().to_string();
        let ak = req.headers().get(BIOS_AUTH_HEADER)?.clone();
        Some(Self::new(method, path, ak))
    }
    fn attach(self, req: &mut http::Request<spacegate_shell::SgBody>) {
        req.headers_mut().insert(BIOS_AUTH_HEADER, self.ak);
    }
    fn detach(req: &mut http::Request<spacegate_shell::SgBody>) -> Option<Self> {
        let ak = req.headers().get(BIOS_AUTH_HEADER)?.clone();
        Some(Self::new(req.method().clone(), req.uri().path().to_string(), ak))
    }
}

impl AsRedisKey for BiosAuth {
    fn as_redis_key(&self, prefix: impl AsRef<str>) -> String {
        format!("{}:{}:{}:{}", prefix.as_ref(), self.method, self.path, self.ak.to_str().unwrap_or("__invalid_ak_header__"))
    }
}

impl ToRedisArgs for BiosAuth {
    fn to_redis_args(&self) -> Vec<Vec<u8>> {
        vec![self.method.as_str().as_bytes().to_vec(), self.path.as_bytes().to_vec()]
    }
    
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + tardis::cache::RedisWrite {
        self.method.as_str().write_redis_args(out);
        self.path.as_str().write_redis_args(out);
    }
}
