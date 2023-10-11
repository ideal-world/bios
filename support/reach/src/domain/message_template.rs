use tardis::basic::dto::TardisContext;
use tardis::chrono::{DateTime, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;

use tardis::db::sea_orm::sea_query::{ColumnDef, Table, TableCreateStatement};
use tardis::db::sea_orm::*;

use crate::fill_by_mod_req;
use crate::{dto::*, fill_by_add_req};
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "reach_msg_template")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Nanoid,
    /// 所有者路径
    #[sea_orm(column_type = "String(Some(255))")]
    pub own_paths: String,
    /// 所有者
    #[sea_orm(column_type = "String(Some(255))")]
    pub owner: String,
    /// 创建时间
    #[sea_orm(column_type = "Timestamp")]
    pub create_time: DateTime<Utc>,
    /// 更新时间
    #[sea_orm(column_type = "Timestamp")]
    pub update_time: DateTime<Utc>,
    /// 资源作用级别
    #[sea_orm(column_name = "scope_level")]
    pub scope_level: i16,
    /// 编码
    #[sea_orm(column_type = "String(Some(255))")]
    pub code: String,
    /// 名称
    #[sea_orm(column_type = "String(Some(255))")]
    pub name: String,
    /// 说明
    #[sea_orm(column_type = "String(Some(2000))")]
    pub note: String,
    /// 图标
    #[sea_orm(column_type = "String(Some(1000))")]
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
    #[sea_orm(column_type = "Text")]
    pub variables: String,
    /// 用户触达等级类型
    #[sea_orm(column_type = "String(Some(255))")]
    pub level_kind: ReachLevelKind,
    /// 主题
    #[sea_orm(column_type = "String(Some(255))")]
    pub topic: String,
    /// 内容
    #[sea_orm(column_type = "Text")]
    pub content: String,
    /// 确认超时时间
    #[sea_orm(column_name = "timeout_sec")]
    pub timeout_sec: i32,
    /// 确认超时策略
    #[sea_orm(column_type = "String(Some(255))")]
    pub timeout_strategy: ReachTimeoutStrategyKind,
    /// 关联的触达通道
    #[sea_orm(column_type = "String(Some(255))")]
    pub rel_reach_channel: ReachChannelKind,
    /// NOTICE
    #[sea_orm(column_type = "String(Some(255))")]
    pub kind: ReachTemplateKind,
    /// 用户触达验证码策略Id
    #[sea_orm(column_type = "String(Some(255))")]
    pub rel_reach_verify_code_strategy_id: String,
    /// 第三方插件-模板Id
    #[sea_orm(column_type = "String(Some(255))")]
    pub sms_template_id: String,
    /// 第三方插件-签名
    #[sea_orm(column_type = "String(Some(255))")]
    pub sms_signature: String,
    /// 第三方插件-短信发送方的号码
    #[sea_orm(column_type = "String(Some(255))")]
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
        fill_by_mod_req! {
            add_req => {
                code,
                name,
                scope_level,
            } model
        }
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
            scope_level,
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
impl ActiveModelBehavior for ActiveModel {}
impl TardisActiveModel for ActiveModel {
    fn fill_ctx(&mut self, ctx: &TardisContext, is_insert: bool) {
        if is_insert {
            self.own_paths = Set(ctx.own_paths.to_string());
            self.owner = Set(ctx.owner.to_string());
        }
    }
    fn create_table_statement(db: DbBackend) -> TableCreateStatement {
        let mut builder = Table::create();
        builder
            .table(Entity.table_ref())
            .if_not_exists()
            .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
            .col(ColumnDef::new(Column::OwnPaths).not_null().string())
            .col(ColumnDef::new(Column::Owner).not_null().string())
            .col(ColumnDef::new(Column::ScopeLevel).not_null().small_integer())
            .col(ColumnDef::new(Column::Code).not_null().string())
            .col(ColumnDef::new(Column::Name).not_null().string())
            .col(ColumnDef::new(Column::Note).not_null().string())
            .col(ColumnDef::new(Column::Icon).not_null().string())
            .col(ColumnDef::new(Column::Sort).not_null().integer())
            .col(ColumnDef::new(Column::Disabled).not_null().boolean())
            .col(ColumnDef::new(Column::Variables).not_null().text())
            .col(ColumnDef::new(Column::LevelKind).not_null().string())
            .col(ColumnDef::new(Column::Topic).not_null().string())
            .col(ColumnDef::new(Column::Content).not_null().text())
            .col(ColumnDef::new(Column::TimeoutSec).not_null().integer())
            .col(ColumnDef::new(Column::TimeoutStrategy).not_null().string())
            .col(ColumnDef::new(Column::RelReachChannel).not_null().string())
            .col(ColumnDef::new(Column::Kind).not_null().string())
            .col(ColumnDef::new(Column::RelReachVerifyCodeStrategyId).not_null().string())
            .col(ColumnDef::new(Column::SmsTemplateId).not_null().string())
            .col(ColumnDef::new(Column::SmsSignature).not_null().string())
            .col(ColumnDef::new(Column::SmsFrom).not_null().string());
        if db == DatabaseBackend::Postgres {
            builder
                .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp_with_time_zone())
                .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp_with_time_zone());
        } else {
            builder
                .engine("InnoDB")
                .character_set("utf8mb4")
                .collate("utf8mb4_0900_as_cs")
                .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp())
                .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string()).timestamp());
        }
        builder
    }
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
