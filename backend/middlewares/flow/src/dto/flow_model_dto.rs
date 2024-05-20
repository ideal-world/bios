use bios_basic::rbum::{
    dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemFilterFetcher, RbumItemRelFilterReq},
    rbum_enumeration::RbumScopeLevelKind,
};
use serde::{Deserialize, Serialize};
use tardis::{
    basic::field::TrimString,
    chrono::{DateTime, Utc},
    db::sea_orm,
    serde_json::Value,
    web::poem_openapi,
    TardisFuns,
};

use super::{
    flow_state_dto::{FlowStateAggResp, FlowStateRelModelExt, FlowStateRelModelModifyReq},
    flow_transition_dto::{FlowTransitionAddReq, FlowTransitionDetailResp, FlowTransitionModifyReq},
};

/// 添加请求
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowModelAddReq {
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: TrimString,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub icon: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub info: Option<String>,
    /// 初始化状态ID
    pub init_state_id: String,
    /// 关联模板ID
    pub rel_template_id: Option<String>,
    /// 绑定的动作
    pub transitions: Option<Vec<FlowTransitionAddReq>>,
    /// 绑定的状态
    pub states: Option<Vec<FlowModelBindStateReq>>,
    /// 是否作为模板使用
    pub template: bool,
    /// 关联父级模型ID
    pub rel_model_id: Option<String>,
    /// 标签
    pub tag: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

/// 修改请求
#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object)]
pub struct FlowModelModifyReq {
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub icon: Option<String>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub info: Option<String>,
    /// 初始化状态ID
    pub init_state_id: Option<String>,
    /// 是否作为模板使用
    pub template: Option<bool>,
    /// 添加动作
    pub add_transitions: Option<Vec<FlowTransitionAddReq>>,
    /// 修改动作
    pub modify_transitions: Option<Vec<FlowTransitionModifyReq>>,
    /// 删除动作
    pub delete_transitions: Option<Vec<String>>,
    /// 绑定状态
    pub bind_states: Option<Vec<FlowModelBindStateReq>>,
    /// 解绑状态
    pub unbind_states: Option<Vec<String>>,
    /// 修改状态
    pub modify_states: Option<Vec<FlowStateRelModelModifyReq>>,
    /// 标签
    pub tag: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

/// 工作流模型概要信息
#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowModelSummaryResp {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub info: String,
    /// 初始化状态ID
    pub init_state_id: String,

    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    /// 标签
    pub tag: String,

    pub disabled: bool,
}

/// 工作流模型详细信息
#[derive(Serialize, Deserialize, Debug, Clone, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowModelDetailResp {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub info: String,
    /// 初始化状态ID
    pub init_state_id: String,
    /// 关联模板ID
    pub rel_template_id: String,
    // 动作信息
    pub transitions: Option<Value>,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    /// 标签
    pub tag: String,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
}

impl FlowModelDetailResp {
    pub fn transitions(&self) -> Vec<FlowTransitionDetailResp> {
        match &self.transitions {
            Some(transitions) => TardisFuns::json.json_to_obj(transitions.clone()).unwrap(),
            None => vec![],
        }
    }
}

/// 工作流模型过滤器
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct FlowModelFilterReq {
    /// 基础过滤
    pub basic: RbumBasicFilterReq,
    /// 标签集合
    pub tags: Option<Vec<String>>,
    /// 关联模板ID
    pub rel_template_id: Option<String>,
    /// 是否作为模板使用
    pub template: Option<bool>,
    pub own_paths: Option<Vec<String>>,
    /// 指定状态ID(用于过滤动作)
    pub specified_state_ids: Option<Vec<String>>,
}

impl RbumItemFilterFetcher for FlowModelFilterReq {
    fn basic(&self) -> &RbumBasicFilterReq {
        &self.basic
    }
    fn rel(&self) -> &Option<RbumItemRelFilterReq> {
        &None
    }
    fn rel2(&self) -> &Option<RbumItemRelFilterReq> {
        &None
    }
}

/// 工作流模型聚合信息
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct FlowModelAggResp {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub info: String,
    /// 初始化状态ID
    pub init_state_id: String,
    /// 关联模板ID
    pub rel_template_id: String,
    /// 绑定的状态
    pub states: Vec<FlowStateAggResp>,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    /// 标签
    pub tag: String,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
}

/// 绑定状态
#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object)]
pub struct FlowModelBindStateReq {
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) id
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) id
    pub state_id: String,
    pub ext: FlowStateRelModelExt,
}

/// 解绑状态
#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object)]
pub struct FlowModelUnbindStateReq {
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) id
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) id
    pub state_id: String,
}

/// 状态排序
#[derive(Serialize, Deserialize, Clone, Debug, Default, poem_openapi::Object)]
pub struct FlowModelSortStatesReq {
    pub sort_states: Vec<FlowModelSortStateInfoReq>,
}

/// 状态排序
#[derive(Serialize, Deserialize, Debug, Default, Clone, poem_openapi::Object)]
pub struct FlowModelSortStateInfoReq {
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) id
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) id
    pub state_id: String,
    /// 排序
    pub sort: i64,
}

/// 创建自定义模板请求
#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object)]
pub struct FlowModelAddCustomModelReq {
    /// 模板ID
    pub proj_template_id: Option<String>,
    /// 绑定模型的对象
    pub bind_model_objs: Vec<FlowModelAddCustomModelItemReq>,
}

/// 绑定模型的对象
#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object)]
pub struct FlowModelAddCustomModelItemReq {
    /// 标签
    pub tag: String,
}

/// 创建自定义模板响应
#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object)]
pub struct FlowModelAddCustomModelResp {
    /// 标签
    pub tag: String,
    /// 创建的模型ID
    pub model_id: Option<String>,
}

/// 获取关联状态的响应
#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object)]
pub struct FlowModelFindRelStateResp {
    /// Associated [flow state](super::flow_state_dto::FlowStateDetailResp) id
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) id
    pub id: String,
    /// Associated [flow state](super::flow_state_dto::FlowStateDetailResp) name
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) name
    pub name: String,
    /// Associated [flow state](super::flow_state_dto::FlowStateDetailResp) color
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) color
    pub color: String,
}
