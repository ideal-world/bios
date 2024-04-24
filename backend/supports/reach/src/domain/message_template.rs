use tardis::basic::dto::TardisContext;
use tardis::chrono::{DateTime, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;

use crate::fill_by_mod_req;
use crate::{dto::*, fill_by_add_req};
use tardis::db::sea_orm::sea_query::{ColumnDef, Table, TableCreateStatement};
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "reach_msg_template")]
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
    /// 资源作用级别
    pub scope_level: Option<i16>,
    /// 编码
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub code: String,
    /// 名称
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub name: String,
    /// 说明
    #[tardis_entity(custom_type = "string", custom_len = "2000")]
    pub note: String,
    /// 图标
    #[tardis_entity(custom_type = "string", custom_len = "1000")]
    pub icon: String,
    /// 排序
    #[sea_orm(column_type = "Integer")]
    pub sort: i32,
    /// 是否禁用
    #[sea_orm(column_type = "Boolean")]
    pub disabled: bool,
    /// 模板变量
    /// - name: 模板字段，对象名.字段名。对于值类型模板，name = x， x为字段名，对于引用类型模板，name = x.y.z, x、y为级联的对象，z为字段名
    /// - required: 是否必须
    /// - defaultValue: 默认值
    #[tardis_entity(custom_type = "text")]
    pub variables: String,
    /// 用户触达等级类型
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub level_kind: ReachLevelKind,
    /// 主题
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub topic: String,
    /// 内容
    #[tardis_entity(custom_type = "text")]
    pub content: String,
    /// 确认超时时间
    pub timeout_sec: i32,
    /// 确认超时策略
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub timeout_strategy: ReachTimeoutStrategyKind,
    /// 关联的触达通道
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub rel_reach_channel: ReachChannelKind,
    /// NOTICE
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub kind: ReachTemplateKind,
    /// 用户触达验证码策略Id
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub rel_reach_verify_code_strategy_id: String,
    /// 第三方插件-模板Id
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub sms_template_id: String,
    /// 第三方插件-签名
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub sms_signature: String,
    /// 第三方插件-短信发送方的号码
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub sms_from: String,
}
impl From<&ReachMessageTemplateAddReq> for ActiveModel {
    fn from(add_req: &ReachMessageTemplateAddReq) -> Self {
        let mut model = ActiveModel {
            create_time: Set(Utc::now()),
            update_time: Set(Utc::now()),
            ..Default::default()
        };
        fill_by_add_req!(add_req => {
            note,
            icon,
            code,
            name,
            sort,
            disabled,
            variables,
            level_kind,
            topic,
            content,
            timeout_sec,
            timeout_strategy,
            rel_reach_channel,
            kind,
            rel_reach_verify_code_strategy_id,
            sms_template_id,
            sms_signature,
            sms_from,
        } model);
        model.scope_level = Set(add_req.scope_level.clone().map(|level| level.to_int()));
        model
    }
}

impl From<&ReachMessageTemplateModifyReq> for ActiveModel {
    fn from(value: &ReachMessageTemplateModifyReq) -> Self {
        let mut active_model: ActiveModel = ActiveModel {
            update_time: Set(Utc::now()),
            ..Default::default()
        };
        fill_by_mod_req!(value => {
            code,
            name,
            note,
            icon,
            sort,
            disabled,
            variables,
            level_kind: Copy,
            topic,
            content,
            timeout_sec,
            timeout_strategy: Copy,
            rel_reach_channel: Copy,
            kind: Copy,
            rel_reach_verify_code_strategy_id,
            sms_template_id,
            sms_signature,
            sms_from,
        } active_model);
        active_model
    }
}
