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

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tardis::log::warn;

use crate::dto::{
    flow_model_dto::FlowModelDetailResp,
    flow_transition_dto::{FlowTransitionActionChangeAgg, FlowTransitionActionChangeKind, TagRelKind},
};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct StateGraph {
    inner: HashMap<String, HashMap<String, Vec<String>>>,
}

impl StateGraph {
    pub fn new(models: &HashMap<String, FlowModelDetailResp>) -> Self {
        let mut state_rels = HashMap::new();
        for (tag, model) in models {
            let tag_rel = state_rels.entry(tag.clone()).or_insert(HashMap::new());
            for trans in model.transitions() {
                let state_rel = tag_rel.entry(trans.from_flow_state_id).or_insert(vec![]);
                state_rel.push(trans.to_flow_state_id);
            }
        }

        Self { inner: state_rels }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TransactionGraph {
    rels: HashMap<(String, String), Vec<(String, String)>>,
}

impl TransactionGraph {
    pub fn new(models: &HashMap<String, FlowModelDetailResp>) -> Self {
        let state_rels = StateGraph::new(models);
        let mut rels = HashMap::new();
        // init
        for (tag, model) in models {
            for trans in model.transitions() {
                rels.insert((format!("{}-{}", tag, trans.from_flow_state_id), format!("{}-{}", tag, trans.to_flow_state_id)), vec![]);
            }
        }
        for (tag, model) in models {
            for trans in model.transitions() {
                let rel = rels.entry((format!("{}-{}", tag, trans.from_flow_state_id), format!("{}-{}", tag, trans.to_flow_state_id))).or_insert(vec![]);
                for action in trans.action_by_post_changes().into_iter().filter(|change| change.kind == FlowTransitionActionChangeKind::State) {
                    if let Some(state_change_info) = FlowTransitionActionChangeAgg::from(action).state_change_info {
                        let obj_tag = match state_change_info.obj_tag_rel_kind.unwrap_or(TagRelKind::Default) {
                            TagRelKind::Default => &state_change_info.obj_tag,
                            TagRelKind::ParentOrSub => tag,
                        };
                        let current_rels_by_tag = state_rels.inner.get(obj_tag).cloned().unwrap_or_default();
                        if let Some(obj_current_state_ids) = state_change_info.obj_current_state_id {
                            for obj_current_state_id in obj_current_state_ids {
                                for target_state in current_rels_by_tag
                                    .get(&obj_current_state_id)
                                    .cloned()
                                    .unwrap_or_default()
                                    .into_iter()
                                    .filter(|target_state| *target_state == state_change_info.changed_state_id)
                                {
                                    rel.push((format!("{}-{}", obj_tag, obj_current_state_id.clone()), format!("{}-{}", obj_tag, target_state.clone())));
                                }
                            }
                        } else {
                            for (original_state_id, target_states) in current_rels_by_tag {
                                if target_states.contains(&state_change_info.changed_state_id) {
                                    rel.push((
                                        format!("{}-{}", obj_tag, original_state_id.clone()),
                                        format!("{}-{}", obj_tag, state_change_info.changed_state_id.clone()),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        for (tag, model) in models {
            for trans in model.transitions() {
                if !trans.action_by_front_changes().is_empty() {
                    for (original_state, tatarget_states) in state_rels.inner.get(tag).cloned().unwrap_or_default() {
                        if tatarget_states.contains(&trans.from_flow_state_id) {
                            let rel = rels.entry((format!("{}-{}", tag, original_state.clone()), format!("{}-{}", tag, trans.from_flow_state_id.clone()))).or_insert(vec![]);
                            rel.push((
                                format!("{}-{}", tag, trans.from_flow_state_id.clone()),
                                format!("{}-{}", tag, trans.to_flow_state_id.clone()),
                            ));
                        }
                    }
                }
            }
        }

        for (from_tran, to_trans) in rels.iter_mut() {
            to_trans.retain(|to_tran| *to_tran != *from_tran);
        }
        Self {
            rels: rels.into_iter().filter(|(_original_trans, target_trans)| !target_trans.is_empty()).collect(),
        }
    }

    pub fn remove_empty_ele(&mut self) {
        let mut rels = self.rels.clone();
        while !self.rels.is_empty() {
            let mut is_modify = false;
            for (from_tran, to_trans) in &rels {
                if to_trans.is_empty() || (to_trans.len() == 1 && to_trans[0] == *from_tran) {
                    self.rels.remove(from_tran);
                    is_modify = true;
                }
            }
            rels.clone_from(&self.rels);
            for (_from_tran, to_trans) in self.rels.iter_mut() {
                for tran in to_trans.clone() {
                    if !rels.clone().contains_key(&tran) {
                        to_trans.retain(|to_tran| *to_tran != tran);
                        is_modify = true;
                    }
                }
            }
            rels.clone_from(&self.rels);
            if !is_modify {
                break;
            }
        }
    }

    pub fn check_state_loop(&self) -> bool {
        let mut state_chains: HashSet<Vec<String>> = HashSet::new();
        for ((from_tran_from_state, from_tran_to_state), to_trans) in &self.rels {
            state_chains.insert(vec![from_tran_from_state.clone(), from_tran_to_state.clone()]);
            for (to_tran_from_state, to_tran_to_state) in to_trans {
                state_chains.insert(vec![to_tran_from_state.clone(), to_tran_to_state.clone()]);
                for state_tran in state_chains.clone() {
                    if state_tran.last().unwrap() == to_tran_from_state {
                        let mut insert_chain = state_tran.clone();
                        insert_chain.push(to_tran_to_state.clone());
                        state_chains.insert(insert_chain);
                    }
                }
            }
        }
        warn!("check state loop state_chains: {:?}", state_chains);
        for state_chain in state_chains {
            let mut tran_chain = vec![];
            let mut from_state = state_chain[0].clone();
            for state in &state_chain[1..] {
                if tran_chain.contains(&(from_state.clone(), state.clone())) {
                    return false;
                }
                tran_chain.push((from_state.clone(), state.clone()));
                from_state.clone_from(state);
            }
        }

        true
    }
}

pub fn check(models: &HashMap<String, FlowModelDetailResp>) -> bool {
    let mut transation_graph = TransactionGraph::new(models);
    warn!("debug before remove: {:?}", transation_graph);
    transation_graph.remove_empty_ele();
    warn!("debug after remove: {:?}", transation_graph);

    transation_graph.check_state_loop()
}
