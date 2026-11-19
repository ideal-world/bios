use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

/// 注册请求：为本次 run 登记运行级续期授权（契约 §5.1）。
#[derive(Clone, Serialize, Deserialize, poem_openapi::Object)]
pub struct IamTokenRenewalRegisterReq {
    pub run_id: String,
    pub secret_key: String,
    /// 申请的授权时长（秒）；缺省或超过 IAM 配置的绝对上限时按上限处理。
    pub ttl_sec: Option<u64>,
}

/// 注册响应；capability 仅本次响应可见一次，服务端只保存 SHA-256 哈希。
#[derive(Clone, Serialize, Deserialize, poem_openapi::Object)]
pub struct IamTokenRenewalRegisterResp {
    pub credential_id: String,
    pub capability: String,
    /// 续期授权的绝对截止时间（RFC3339），兑换不能向后延长。
    pub expires_at: String,
}

/// 兑换请求：由受信宿主携带 `run_id + capability` 调用（契约 §5.2）。
#[derive(Clone, Serialize, Deserialize, poem_openapi::Object)]
pub struct IamTokenRenewalExchangeReq {
    pub run_id: String,
    pub capability: String,
}

/// 兑换响应；`expires_at` 为本次签发 token 的到期时间，不是授权的绝对期限。
#[derive(Clone, Serialize, Deserialize, poem_openapi::Object)]
pub struct IamTokenRenewalExchangeResp {
    pub access_token: String,
    pub expires_at: String,
}

/// 吊销请求：由受信宿主携带 `run_id + capability` 调用（契约 §5.3）。
#[derive(Clone, Serialize, Deserialize, poem_openapi::Object)]
pub struct IamTokenRenewalRevokeReq {
    pub run_id: String,
    pub capability: String,
}

// 敏感凭证不得通过 Debug 输出。
macro_rules! redacted_debug {
    ($($ty:ty),*) => { $(impl std::fmt::Debug for $ty {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(concat!(stringify!($ty), " { [REDACTED] }"))
        }
    })* };
}
redacted_debug!(IamTokenRenewalRegisterReq, IamTokenRenewalRegisterResp, IamTokenRenewalExchangeReq, IamTokenRenewalExchangeResp, IamTokenRenewalRevokeReq);
