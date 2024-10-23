use bios_basic::rbum::{
    dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemFilterFetcher, RbumItemRelFilterReq},
    rbum_enumeration::RbumScopeLevelKind,
};
use serde::{Deserialize, Serialize};
use tardis::{
    basic::field::TrimString,
    chrono::{DateTime, Utc},
    db::sea_orm::{self, prelude::*, EnumIter},
    serde_json::Value,
    web::poem_openapi,
    TardisFuns,
};

use super::{
    flow_model_dto::FlowModelBindStateReq,
    flow_state_dto::{FlowStateAddReq, FlowStateAggResp},
    flow_transition_dto::{FlowTransitionAddReq, FlowTransitionModifyReq},
};

/// 版本状态
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, EnumIter, sea_orm::DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(255))")]
pub enum FlowModelVesionState {
    #[default]
    /// 启用中
    #[sea_orm(string_value = "enabled")]
    Enabled,
    /// 已关闭
    #[sea_orm(string_value = "disabled")]
    Disabled,
    /// 编辑中
    #[sea_orm(string_value = "editing")]
    Editing,
}

/// 添加请求
#[derive(Clone, Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowModelVersionAddReq {
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: TrimString,
    /// 关联的模型ID
    pub rel_model_id: Option<String>,
    /// 配置状态节点
    pub bind_states: Option<Vec<FlowModelVersionBindState>>,
    /// 版本状态
    pub status: FlowModelVesionState,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

/// 模型绑定状态节点
#[derive(Clone, Serialize, Deserialize, Debug, Default, poem_openapi::Object)]
pub struct FlowModelVersionBindState {
    /// 若存在则表示，绑定已有状态节点
    pub exist_state: Option<FlowModelBindStateReq>,
    /// 若存在则表示，新建状态节点
    pub new_state: Option<FlowStateAddReq>,
    /// 添加动作
    pub add_transitions: Option<Vec<FlowTransitionAddReq>>,
    /// 修改动作
    pub modify_transitions: Option<Vec<FlowTransitionModifyReq>>,
    /// 删除动作
    pub delete_transitions: Option<Vec<String>>,
    /// 是否为初始节点
    pub is_init: bool,
}

/// 添加请求
#[derive(Clone, Serialize, Deserialize, Debug, Default, poem_openapi::Object)]
pub struct FlowModelVersionModifyReq {
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: Option<TrimString>,
    // 状态信息
    pub bind_states: Option<Vec<FlowModelVersionBindState>>,
    /// 定义每个模块的初始状态
    pub init_state_id: Option<String>,
    /// 版本状态
    pub status: Option<FlowModelVesionState>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

// FlowModelSummaryResp, FlowModelDetailResp, FlowModelFilterReq
/// 工作流版本模型概要信息
#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowModelVersionSummaryResp {
    pub id: String,
    pub name: String,
    /// 关联的模型ID
    pub rel_model_id: String,
    /// Initial state / 初始状态
    ///
    /// Define the initial state of each model
    /// 定义每个模块的初始状态
    pub init_state_id: String,

    /// 状态 启用中 已关闭
    pub status: FlowModelVesionState,

    pub owner: String,
    pub own_paths: String,

    /// Creation time / 创建时间
    pub create_time: DateTime<Utc>,
    /// 创建者信息
    pub create_by: String,
    /// 更新时间
    pub update_time: DateTime<Utc>,
    /// 修改人信息
    pub update_by: String,
}

/// 工作流模型详细信息
#[derive(Serialize, Deserialize, Debug, Clone, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowModelVersionDetailResp {
    pub id: String,
    pub name: String,
    /// 初始化状态ID
    pub init_state_id: String,
    /// 关联父级模型ID
    pub rel_model_id: String,
    // 状态信息
    pub states: Option<Value>,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
}

impl FlowModelVersionDetailResp {
    pub fn states(&self) -> Vec<FlowStateAggResp> {
        match &self.states {
            Some(states) => TardisFuns::json.json_to_obj(states.clone()).unwrap(),
            None => vec![],
        }
    }
}

/// 工作流模型版本过滤器
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct FlowModelVersionFilterReq {
    /// 基础过滤
    pub basic: RbumBasicFilterReq,

    pub own_paths: Option<Vec<String>>,
    pub status: Option<Vec<FlowModelVesionState>>,
    /// 关联模型ID
    pub rel_model_ids: Option<Vec<String>>,

    pub rel: Option<RbumItemRelFilterReq>,
    pub rel2: Option<RbumItemRelFilterReq>,
}

impl RbumItemFilterFetcher for FlowModelVersionFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
    fn rel(&self) -> &Option<RbumItemRelFilterReq> {
        &self.rel
    }
    fn rel2(&self) -> &Option<RbumItemRelFilterReq> {
        &self.rel2
    }
}
