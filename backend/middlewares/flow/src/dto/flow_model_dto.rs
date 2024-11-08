use std::{collections::HashMap, default};

use bios_basic::rbum::{
    dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemFilterFetcher, RbumItemRelFilterReq},
    rbum_enumeration::RbumScopeLevelKind,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tardis::{
    basic::field::TrimString,
    chrono::{DateTime, Utc},
    db::sea_orm::{self, prelude::*},
    serde_json::Value,
    web::poem_openapi,
    TardisFuns,
};

use super::{
    flow_model_version_dto::{FlowModelVersionAddReq, FlowModelVersionBindState, FlowModelVersionModifyReq, FlowModelVesionState},
    flow_state_dto::{FLowStateIdAndName, FlowStateAggResp, FlowStateRelModelExt},
    flow_transition_dto::{FlowTransitionAddReq, FlowTransitionDetailResp},
};

/// 添加请求
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowModelAddReq {
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: TrimString,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub icon: Option<String>,
    #[oai(validator(max_length = "2000"))]
    pub info: Option<String>,
    /// 工作流模型类型
    pub kind: FlowModelKind,
    /// 工作流模型状态
    pub status: FlowModelStatus,
    /// 关联模板ID（目前可能是页面模板ID，或者是项目模板ID）
    pub rel_template_ids: Option<Vec<String>>,
    /// 关联动作ID（触发当前工作流的动作，若为空则默认表示新建数据时触发）
    pub rel_transition_ids: Option<Vec<String>>,
    /// 创建的可用版本
    pub add_version: Option<FlowModelVersionAddReq>,
    pub current_version_id: Option<String>,
    /// 是否作为模板使用
    pub template: bool,
    /// 是否作为主流程
    pub main: bool,
    /// 关联父级模型ID
    pub rel_model_id: Option<String>,
    /// 标签
    pub tag: Option<String>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,
}

impl From<FlowModelDetailResp> for FlowModelAddReq {
    fn from(value: FlowModelDetailResp) -> Self {
        let mut add_transitions = vec![];
        for transition in value.transitions() {
            add_transitions.push(FlowTransitionAddReq::from(transition));
        }
        let states = value
            .states()
            .into_iter()
            .map(|state| FlowModelVersionBindState {
                exist_state: Some(FlowModelBindStateReq {
                    state_id: state.id.clone(),
                    ext: state.ext,
                }),
                new_state: None,
                add_transitions: Some(add_transitions.clone().into_iter().filter(|tran| tran.from_flow_state_id == state.id).collect_vec()),
                modify_transitions: None,
                delete_transitions: None,
                is_init: value.init_state_id == state.id,
            })
            .collect_vec();
        Self {
            name: value.name.as_str().into(),
            icon: Some(value.icon.clone()),
            info: Some(value.info.clone()),
            kind: value.kind,
            status: value.status,
            rel_transition_ids: None,
            rel_template_ids: Some(value.rel_template_ids.clone()),
            add_version: Some(FlowModelVersionAddReq {
                name: value.name.as_str().into(),
                rel_model_id: None,
                bind_states: Some(states),
                status: FlowModelVesionState::Enabled,
                scope_level: Some(value.scope_level.clone()),
                disabled: Some(value.disabled),
            }),
            current_version_id: None,
            template: value.template,
            main: value.main,
            rel_model_id: None,
            tag: Some(value.tag.clone()),
            scope_level: Some(value.scope_level),
            disabled: Some(value.disabled),
        }
    }
}

/// 工作流模型类型
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, EnumIter, sea_orm::DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(255))")]
pub enum FlowModelKind {
    #[sea_orm(string_value = "as_template")]
    AsTemplate,
    #[sea_orm(string_value = "as_model")]
    AsModel,
    #[sea_orm(string_value = "as_template_and_as_model")]
    AsTemplateAndAsModel,
}

/// 工作流模型状态
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum, EnumIter, sea_orm::DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(255))")]
pub enum FlowModelStatus {
    #[default]
    #[sea_orm(string_value = "enabled")]
    Enabled,
    #[sea_orm(string_value = "disabled")]
    Disabled,
}

/// 修改请求
#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object, Clone)]
pub struct FlowModelModifyReq {
    #[oai(validator(min_length = "2", max_length = "200"))]
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "2000"))]
    pub icon: Option<String>,
    #[oai(validator(max_length = "2000"))]
    pub info: Option<String>,
    /// 是否作为模板使用
    pub template: Option<bool>,
    /// 状态
    pub status: Option<FlowModelStatus>,
    /// 当前版本ID
    pub current_version_id: Option<String>,
    /// 修改版本
    pub modify_version: Option<FlowModelVersionModifyReq>,
    /// 标签
    pub tag: Option<String>,
    /// 关联模板ID（目前可能是页面模板ID，或者是项目模板ID）
    pub rel_template_ids: Option<Vec<String>>,
    /// 关联父级工作流模板ID
    pub rel_model_id: Option<String>,

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
    pub init_state_id: String,
    pub current_version_id: String,
    pub owner: String,
    pub own_paths: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    /// 标签
    pub tag: String,

    pub disabled: bool,
    pub status: FlowModelStatus,

    pub states: Value,
    /// 关联动作
    pub rel_transition: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowModelRelTransitionExt {
    pub id: String,
    pub name: String,
    pub from_flow_state_name: String,
}

/// 工作流模型详细信息
#[derive(Serialize, Deserialize, Debug, Clone, poem_openapi::Object, sea_orm::FromQueryResult)]
pub struct FlowModelDetailResp {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub info: String,
    pub kind: FlowModelKind,
    pub status: FlowModelStatus,
    /// 是否作为模板使用
    pub template: bool,
    /// 是否主流程
    pub main: bool,

    pub init_state_id: String,
    pub current_version_id: String,
    /// 关联父级模型ID
    pub rel_model_id: String,
    /// 关联模板ID（目前可能是页面模板ID，或者是项目模板ID）
    pub rel_template_ids: Vec<String>,
    // 动作信息
    pub transitions: Option<Value>,
    // 状态信息
    pub states: Option<Value>,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    /// 标签
    pub tag: String,

    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
    /// 关联动作
    pub rel_transition: Option<Value>,
}

impl FlowModelDetailResp {
    pub fn transitions(&self) -> Vec<FlowTransitionDetailResp> {
        match &self.transitions {
            Some(transitions) => TardisFuns::json.json_to_obj(transitions.clone()).unwrap(),
            None => vec![],
        }
    }

    pub fn states(&self) -> Vec<FlowStateAggResp> {
        match &self.states {
            Some(states) => TardisFuns::json.json_to_obj(states.clone()).unwrap_or_default(),
            None => vec![],
        }
    }

    pub fn rel_transition(&self) -> Option<FlowModelRelTransitionExt> {
        self.rel_transition.clone().map(|rel_transition| TardisFuns::json.json_to_obj(rel_transition.clone()).unwrap())
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

    pub kinds: Option<Vec<FlowModelKind>>,
    pub status: Option<FlowModelStatus>,
    /// 是否作为模板使用
    pub template: Option<bool>,
    /// 是否是主流程
    pub main: Option<bool>,
    pub own_paths: Option<Vec<String>>,
    /// 指定状态ID(用于过滤动作)
    pub specified_state_ids: Option<Vec<String>>,
    /// 关联模型ID
    pub rel_model_ids: Option<Vec<String>>,
    /// 关联模板ID
    pub rel_template_id: Option<String>,

    pub rel: Option<RbumItemRelFilterReq>,
    pub rel2: Option<RbumItemRelFilterReq>,
}

impl RbumItemFilterFetcher for FlowModelFilterReq {
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

/// 工作流模型聚合信息
#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct FlowModelAggResp {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub info: String,
    /// 是否作为模板使用
    pub template: bool,
    /// 关联父级模型ID
    pub rel_model_id: String,
    pub init_state_id: String,
    /// 关联模板ID（目前可能是页面模板ID，或者是项目模板ID）
    pub rel_template_ids: Vec<String>,
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
#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object, Clone)]
pub struct FlowModelBindStateReq {
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) id
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) id
    pub state_id: String,
    pub ext: FlowStateRelModelExt,
}

impl From<FlowStateAggResp> for FlowModelBindStateReq {
    fn from(value: FlowStateAggResp) -> Self {
        Self {
            state_id: value.id,
            ext: value.ext,
        }
    }
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
    /// 关联模板ID
    pub rel_template_id: Option<String>,
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

/// 工作流关联操作类型
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, poem_openapi::Enum)]
pub enum FlowModelAssociativeOperationKind {
    #[default]
    Reference,
    Copy,
    ReferenceOrCopy,
}

/// 创建或引用模型请求
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowModelCopyOrReferenceReq {
    /// 关联的模型ID列表
    pub rel_model_ids: HashMap<String, String>,
    /// 关联的模板ID
    pub rel_template_id: Option<String>,
    /// 关联操作
    pub op: FlowModelAssociativeOperationKind,
}

/// 创建或引用模型请求
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowModelSingleCopyOrReferenceReq {
    /// 关联的模型ID列表
    pub tag: String,
    /// 关联的模型ID
    pub rel_model_id: String,
    /// 关联操作
    pub op: FlowModelAssociativeOperationKind,
}

/// 创建或引用模型请求
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowModelCopyOrReferenceCiReq {
    /// 关联的模板ID
    pub rel_template_id: Option<String>,
    /// 关联操作
    pub op: FlowModelAssociativeOperationKind,
}

/// 检查关联模板请求
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowModelExistRelByTemplateIdsReq {
    /// 关联的模板中的tag信息
    pub rel_tag_by_template_ids: HashMap<String, Vec<String>>,
    /// 需要支持关联的tag
    pub support_tags: Vec<String>,
}

/// 获取关联的模型名请求
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowModelFindRelNameByTemplateIdsReq {
    /// 关联的模板ID
    pub rel_template_ids: Vec<String>,
}