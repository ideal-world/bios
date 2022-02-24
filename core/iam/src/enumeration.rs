use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Display, Debug, Deserialize, Serialize)]
pub enum RbumScopeKind {
    /// 标签级
    /// 表明只这个标签可用
    #[display(fmt = "TAG")]
    TAG,
    /// 应用级
    /// 表明在应用内共享
    #[display(fmt = "APP")]
    APP,
    /// 租户级
    /// 表明在租户内共享
    #[display(fmt = "TENANT")]
    TENANT,
    /// 系统级
    /// 表明整个系统共享
    #[display(fmt = "GLOBAL")]
    GLOBAL,
}
