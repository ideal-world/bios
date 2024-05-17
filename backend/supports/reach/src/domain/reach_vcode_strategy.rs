use crate::dto::*;
use crate::fill_by_add_req;
use tardis::chrono::{self, DateTime, Utc};
use tardis::db::sea_orm;

use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "reach_vcode_strategy")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[tardis_entity(custom_type = "string")]
    pub id: Nanoid,
    /// 所有者路径
    #[fill_ctx(fill = "own_paths")]
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub own_paths: String,
    /// 所有者
    #[fill_ctx]
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub owner: String,
    /// 创建时间
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub create_time: DateTime<Utc>,
    /// 更新时间
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub update_time: DateTime<Utc>,
    /// 最大错误次数
    #[sea_orm(column_type = "Integer")]
    pub max_error_times: i32,
    /// 过期时间
    #[sea_orm(column_type = "Integer")]
    pub expire_sec: i32,
    /// 验证码长度
    #[sea_orm(column_type = "Integer")]
    pub length: i32,
    /// TODO remove? 使用场景?
    /// 关联触达集合ID
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub rel_reach_set_id: String,
    /// 资源作用级别
    #[sea_orm(column_name = "scope_level")]
    pub scope_level: Option<i16>,
}

impl From<&ReachVCodeStrategyAddReq> for ActiveModel {
    fn from(value: &ReachVCodeStrategyAddReq) -> Self {
        let mut model = ActiveModel {
            update_time: Set(chrono::Utc::now()),
            create_time: Set(chrono::Utc::now()),
            ..Default::default()
        };
        fill_by_add_req!(
            value => {
                max_error_times,
                expire_sec,
                length,
                rel_reach_set_id,
            } model
        );
        model
    }
}

impl From<&ReachVCodeStrategyModifyReq> for ActiveModel {
    fn from(value: &ReachVCodeStrategyModifyReq) -> Self {
        let mut model = ActiveModel {
            update_time: Set(chrono::Utc::now()),
            ..Default::default()
        };
        fill_by_add_req!(
            value => {
                max_error_times,
                expire_sec,
                length,
            } model
        );
        model
    }
}
