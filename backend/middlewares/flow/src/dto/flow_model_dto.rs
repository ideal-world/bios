use std::{collections::HashMap, fmt};

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
    serde_json::json,
    serde_json::Value,
    web::poem_openapi,
    TardisFuns,
};

use super::{
    flow_cond_dto::BasicQueryCondInfo,
    flow_model_version_dto::{FlowModelVersionAddReq, FlowModelVersionBindState, FlowModelVersionModifyReq, FlowModelVesionState},
    flow_state_dto::{FlowStateAddReq, FlowStateAggResp, FlowStateRelModelExt},
    flow_transition_dto::{FlowTransitionAddReq, FlowTransitionDetailResp},
};

/// 添加请求
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowModelAddReq {
    pub id: Option<String>,
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

    /// 满足条件时，触发该流程
    pub front_conds: Option<Vec<Vec<BasicQueryCondInfo>>>,

    pub scope_level: Option<RbumScopeLevelKind>,
    pub disabled: Option<bool>,

    pub data_source: Option<String>,
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
                bind_new_state: None,
                add_transitions: Some(add_transitions.clone().into_iter().filter(|tran| tran.from_flow_state_id == state.id).collect_vec()),
                modify_transitions: None,
                delete_transitions: None,
                is_init: value.init_state_id == state.id,
            })
            .collect_vec();
        let rel_transition_ids = value.rel_transitions().clone().map(|rel_transitions| rel_transitions.into_iter().map(|tran| tran.id).collect_vec());
        let front_conds = value.front_conds();
        Self {
            id: None,
            name: value.name.as_str().into(),
            icon: Some(value.icon.clone()),
            info: Some(value.info.clone()),
            kind: value.kind,
            status: value.status,
            rel_transition_ids,
            rel_template_ids: Some(value.rel_template_ids.clone()),
            add_version: Some(FlowModelVersionAddReq {
                id: None,
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
            front_conds,
            rel_model_id: None,
            tag: Some(value.tag.clone()),
            scope_level: Some(value.scope_level),
            disabled: Some(value.disabled),
            data_source: value.data_source,
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

    /// 满足条件时，触发该流程
    pub front_conds: Option<Vec<Vec<BasicQueryCondInfo>>>,

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
    pub rel_model_id: String,
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
    pub rel_transitions: Option<Value>,
}

impl From<FlowModelAggResp> for FlowModelSummaryResp {
    fn from(value: FlowModelAggResp) -> Self {
        Self {
            id: value.id,
            name: value.name,
            icon: value.icon,
            info: value.info,
            init_state_id: value.init_state_id,
            rel_model_id: value.rel_model_id,
            current_version_id: value.current_version_id,
            owner: value.owner,
            own_paths: value.own_paths,
            create_time: value.create_time,
            update_time: value.update_time,
            tag: value.tag,
            disabled: value.disabled,
            status: value.status,
            states: json!(value.states),
            rel_transitions: value.rel_transitions.map(|rel_transitions| json!(rel_transitions)),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Default, Debug, poem_openapi::Object, sea_orm::FromJsonQueryResult)]
pub struct FlowModelRelTransitionExt {
    pub id: String,
    pub name: String,
    pub from_flow_state_name: String,
    pub to_flow_state_name: Option<String>,
}

/// 关联动作类型
#[derive(PartialEq, Default, Debug, Clone)]
pub enum FlowModelRelTransitionKind {
    #[default]
    Edit,
    Delete,
    Related,
    Review,
    Transfer(FlowModelRelTransitionExt),
}

impl fmt::Display for FlowModelRelTransitionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Edit => write!(f, "编辑"),
            Self::Delete => write!(f, "删除"),
            Self::Related => write!(f, "关联"),
            Self::Review => write!(f, "评审"),
            Self::Transfer(tran) => {
                write!(f, "{}({})", tran.name, tran.from_flow_state_name)
            }
        }
    }
}

impl From<FlowModelRelTransitionExt> for FlowModelRelTransitionKind {
    fn from(value: FlowModelRelTransitionExt) -> Self {
        match value.id.as_str() {
            "__EDIT__" => Self::Edit,
            "__DELETE__" => Self::Delete,
            "__REQRELATED__" => Self::Related,
            "__TASKRELATED__" => Self::Related,
            "__REVIEW__" => Self::Review,
            _ => Self::Transfer(value),
        }
    }
}

impl FlowModelRelTransitionKind {
    pub fn log_text(&self) -> String {
        match self {
            Self::Edit => "编辑审批".to_string(),
            Self::Delete => "删除审批".to_string(),
            Self::Related => "关联审批".to_string(),
            Self::Review => "评审审批".to_string(),
            Self::Transfer(tran) => format!("{}({})", tran.name, tran.from_flow_state_name).to_string(),
        }
    }
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
    /// 标签
    pub tag: String,

    /// 关联动作
    pub rel_transitions: Option<Value>,
    /// 满足条件时，触发该流程
    pub front_conds: Option<Value>,

    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub scope_level: RbumScopeLevelKind,
    pub disabled: bool,
    pub data_source: Option<String>,
}

impl FlowModelDetailResp {
    pub fn transitions(&self) -> Vec<FlowTransitionDetailResp> {
        match &self.transitions {
            Some(transitions) => TardisFuns::json.json_to_obj(transitions.clone()).unwrap_or_default(),
            None => vec![],
        }
    }

    pub fn states(&self) -> Vec<FlowStateAggResp> {
        match &self.states {
            Some(states) => TardisFuns::json.json_to_obj(states.clone()).unwrap_or_default(),
            None => vec![],
        }
    }

    pub fn rel_transitions(&self) -> Option<Vec<FlowModelRelTransitionExt>> {
        self.rel_transitions.clone().map(|rel_transitions| TardisFuns::json.json_to_obj(rel_transitions).unwrap_or_default())
    }

    pub fn front_conds(&self) -> Option<Vec<Vec<BasicQueryCondInfo>>> {
        self.front_conds.clone().map(|front_conds| TardisFuns::json.json_to_obj(front_conds).unwrap_or_default())
    }

    pub fn create_modify_req(self) -> FlowModelModifyReq {
        let front_conds = self.front_conds();
        FlowModelModifyReq {
            name: Some(self.name.as_str().into()),
            icon: Some(self.icon.clone()),
            info: Some(self.info.clone()),
            status: Some(self.status),
            template: Some(self.template),
            rel_template_ids: Some(self.rel_template_ids.clone()),
            modify_version: Some(FlowModelVersionModifyReq {
                name: Some(self.name.as_str().into()),
                bind_states: None,   // todo
                unbind_states: None, // todo
                delete_states: None, // todo
                modify_states: None, // todo
                status: None,
                init_state_id: Some(self.init_state_id),
                scope_level: Some(self.scope_level.clone()),
                disabled: Some(self.disabled),
            }),
            current_version_id: Some(self.current_version_id.clone()),
            front_conds,
            rel_model_id: Some(self.rel_model_id.clone()),
            tag: Some(self.tag.clone()),
            scope_level: Some(self.scope_level),
            disabled: Some(self.disabled),
        }
    }

    pub fn create_add_req(self) -> FlowModelAddReq {
        let mut add_transitions = vec![];
        for transition in self.transitions() {
            let mut add_transitions_req = FlowTransitionAddReq::from(transition.clone());
            add_transitions_req.id = Some(transition.id.clone());
            add_transitions.push(add_transitions_req);
        }
        let states = self
            .states()
            .into_iter()
            .map(|state| FlowModelVersionBindState {
                exist_state: Some(FlowModelBindStateReq {
                    state_id: state.id.clone(),
                    ext: state.ext,
                }),
                bind_new_state: None,
                add_transitions: Some(add_transitions.clone().into_iter().filter(|tran| tran.from_flow_state_id == state.id).collect_vec()),
                modify_transitions: None,
                delete_transitions: None,
                is_init: self.init_state_id == state.id,
            })
            .collect_vec();
        let rel_transition_ids = self.rel_transitions().clone().map(|rel_transitions| rel_transitions.into_iter().map(|tran| tran.id).collect_vec());
        let front_conds = self.front_conds();
        FlowModelAddReq {
            id: Some(self.id.clone()),
            name: self.name.as_str().into(),
            icon: Some(self.icon.clone()),
            info: Some(self.info.clone()),
            kind: self.kind,
            status: self.status,
            rel_transition_ids,
            rel_template_ids: Some(self.rel_template_ids.clone()),
            add_version: Some(FlowModelVersionAddReq {
                id: Some(self.current_version_id.clone()),
                name: self.name.as_str().into(),
                rel_model_id: Some(self.id.clone()),
                bind_states: Some(states),
                status: FlowModelVesionState::Enabled,
                scope_level: Some(self.scope_level.clone()),
                disabled: Some(self.disabled),
            }),
            current_version_id: Some(self.current_version_id.clone()),
            template: self.template,
            main: self.main,
            front_conds,
            rel_model_id: Some(self.rel_model_id.clone()),
            tag: Some(self.tag.clone()),
            scope_level: Some(self.scope_level),
            disabled: Some(self.disabled),
            data_source: self.data_source,
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

    pub kinds: Option<Vec<FlowModelKind>>,
    pub status: Option<FlowModelStatus>,
    /// 是否作为模板使用
    pub template: Option<bool>,
    /// 是否是主流程
    pub main: Option<bool>,
    pub own_paths: Option<Vec<String>>,
    pub data_source: Option<String>,
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
    pub status: FlowModelStatus,
    /// 是否作为模板使用
    pub template: bool,
    /// 关联父级模型ID
    pub rel_model_id: String,
    pub init_state_id: String,
    pub current_version_id: String,
    pub edit_version_id: String,
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
    /// 是否作为主流程
    pub main: bool,
    /// 关联动作
    pub rel_transitions: Option<Vec<FlowModelRelTransitionExt>>,
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

/// 绑定状态
#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object, Clone)]
pub struct FlowModelBindNewStateReq {
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) id
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) id
    pub new_state: FlowStateAddReq,
    pub ext: FlowStateRelModelExt,
}

/// 解绑状态
#[derive(Serialize, Deserialize, Debug, Default, poem_openapi::Object)]
pub struct FlowModelUnbindStateReq {
    /// Associated [flow_state](super::flow_state_dto::FlowStateDetailResp) id
    ///
    /// 关联的[工作流状态](super::flow_state_dto::FlowStateDetailResp) id
    pub state_id: String,
    /// 新的状态ID
    pub new_state_id: String,
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
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, poem_openapi::Enum)]
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
    /// 切换模板时，状态更新映射
    pub update_states: Option<HashMap<String, HashMap<String, String>>>,
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

/// 修改当前参数列表
#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowModelSyncModifiedFieldReq {
    pub rel_template_id: Option<String>,
    pub tag: String,
    /// 参数列表
    pub add_fields: Option<Vec<String>>,
    pub delete_fields: Option<Vec<String>>,
}

/// 状态排序
#[derive(Serialize, Deserialize, Clone, Debug, Default, poem_openapi::Object)]
pub struct FlowModelFIndOrCreatReq {
    pub rel_template_id: String,
    pub tags: Vec<String>,
    pub op: FlowModelAssociativeOperationKind,
    pub data_source: Option<String>,
}

/// 初始化复制模型
#[derive(Serialize, Deserialize, Clone, Debug, Default, poem_openapi::Object)]
pub struct FlowModelInitCopyReq {
    pub rel_template_ids: Vec<String>,
    pub own_path: Vec<String>,
    pub rel_model_id: String,
    pub sync_inst: bool,
}
