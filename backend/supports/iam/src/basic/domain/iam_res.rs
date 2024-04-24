use tardis::db::sea_orm;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "iam_res")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[index]
    pub kind: i16,
    pub icon: String,
    pub sort: i64,
    // 资源方法 例如：*、GET、POST、PUT、DELETE
    pub method: String,
    // 是否隐藏
    pub hide: bool,
    // 资源动作 例如：*、list、create、update、delete
    pub action: String,
    // Whether request is encrypted or not / 请求是否加密
    pub crypto_req: bool,
    // Whether response is encrypted or not / 响应是否加密
    pub crypto_resp: bool,
    // Is secondary certification required / 是否需要二次认证
    pub double_auth: bool,
    /// Secondary Authentication Message / 二次认证消息
    pub double_auth_msg: String,
    // 是否需要验证登陆
    pub need_login: bool,

    pub ext: String,

    #[fill_ctx(fill = "own_paths")]
    pub own_paths: String,
}
