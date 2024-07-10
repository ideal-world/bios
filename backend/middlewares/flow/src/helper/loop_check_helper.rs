//! Infinite loop check helper
//!
//! 按照tag,构造有向图结构，图以状态为节点，以动作为边。判断状态流转是否存在无限循环的问题就转化为判断图中是否存在无限循环的问题。
//! 
//! 实际的数据结构类似于：
//! {
//!     "req": {
//!         "stateA": ["stateB"],
//!         "stateB": ["stateC"],
//!         "stateC": []
//!     },
//!     "task": {
//!         "stateC": []
//!     },
//! }

use std::collections::HashMap;

use crate::dto::flow_model_dto::FlowModelDetailResp;

#[derive(Clone, Debug, Default)]
struct GraphAggs {
    inner: HashMap<String, StateGraph>,
}

#[derive(Clone, Debug, Default)]
struct StateGraph {
    is_modify: bool,
    data: HashMap<String, Vec<String>>,
}

impl GraphAggs {
    pub fn new(models: HashMap<String, FlowModelDetailResp>) -> Self {
        let mut inner = HashMap::new();
        for (tag, model) in models {
            // inner.insert(tag, );
        }
        GraphAggs {
            inner
        }
    }
}

impl StateGraph {
    // pub fn new(model: FlowModelDetailResp) -> Self {

    // }
} 