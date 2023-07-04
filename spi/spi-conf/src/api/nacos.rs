mod v1;
mod v2;
pub use self::v1::*;
mod v2;
pub use self::v2::*;

pub type ConfNacosApi = (ConfNacosV1Api, ConfNacosV2Api);
