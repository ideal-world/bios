use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{constants::RES_CONTAINER, mini_tardis::basic::TardisResult};

pub(crate) fn add_res(res_action: &str, res_uri: &str, need_crypto_req: bool, need_crypto_resp: bool, need_double_auth: bool) -> TardisResult<()> {
    web_sys::console::log_1(&format!("[BIOS.RES] Add res [{res_action}] {res_uri}.").into());
    let res_action = res_action.to_lowercase();
    let res_items = parse_uri(res_uri)?;
    let mut res_container = RES_CONTAINER.write()?;
    if res_container.is_none() {
        *res_container = Some(ResContainerNode::new());
    }
    let mut res_container_node = res_container.as_mut().unwrap();
    for res_item in res_items.into_iter() {
        if !res_container_node.has_child(&res_item) {
            res_container_node.insert_child(&res_item);
        }
        res_container_node = res_container_node.get_child_mut(&res_item);
        if res_item == "$" {
            res_container_node.insert_leaf(&res_action, need_crypto_req, need_crypto_resp, need_double_auth);
        }
    }
    Ok(())
}

pub(crate) fn match_res(res_action: &str, res_uri: &str) -> TardisResult<Vec<ResContainerLeafInfo>> {
    let res_action = res_action.to_lowercase();
    let mut res_items = parse_uri(res_uri)?;
    // remove $ node;
    res_items.remove(res_items.len() - 1);
    let mut matched_uris = vec![];
    let res_container = RES_CONTAINER.read()?;
    do_match_res(&res_action, res_container.as_ref().unwrap(), &res_items, false, &mut matched_uris);
    Ok(matched_uris)
}

fn do_match_res(res_action: &str, res_container: &ResContainerNode, res_items: &Vec<String>, multi_wildcard: bool, matched_uris: &mut Vec<ResContainerLeafInfo>) {
    // TODO "res_items[0] == "?"" approach will ignore the query, there needs to be a better way
    if res_container.has_child("$") && (res_items.is_empty() || multi_wildcard || res_items[0] == "?") {
        // matched
        if let Some(leaf_node) = res_container.get_child("$").get_child_opt(res_action) {
            matched_uris.push(leaf_node.get_leaf_info());
        }
        if let Some(leaf_node) = res_container.get_child("$").get_child_opt("*") {
            matched_uris.push(leaf_node.get_leaf_info());
        }
        return;
    }
    if res_items.is_empty() {
        // un-matched
        return;
    }
    let curr_items = &res_items[0];
    let next_items = &res_items[1..].to_vec();
    if let Some(next_res_container) = res_container.get_child_opt(curr_items) {
        do_match_res(res_action, next_res_container, next_items, false, matched_uris);
    }
    if let Some(next_res_container) = res_container.get_child_opt("*") {
        do_match_res(res_action, next_res_container, next_items, false, matched_uris);
    }
    if let Some(next_res_container) = res_container.get_child_opt("**") {
        do_match_res(res_action, next_res_container, next_items, true, matched_uris);
    }
    if multi_wildcard {
        do_match_res(res_action, res_container, next_items, true, matched_uris);
    }
}

fn parse_uri(res_uri: &str) -> TardisResult<Vec<String>> {
    let mut res_uri = res_uri.split("/").map(|i| i.to_string()).collect::<Vec<String>>();
    res_uri.push("$".to_string());
    Ok(res_uri)
}

pub(crate) struct ResContainerNode {
    children: Option<HashMap<String, ResContainerNode>>,
    leaf_info: Option<ResContainerLeafInfo>,
}

impl ResContainerNode {
    pub fn new() -> Self {
        ResContainerNode {
            children: Some(HashMap::new()),
            leaf_info: None,
        }
    }
    pub fn has_child(&self, key: &str) -> bool {
        self.children.as_ref().map(|n| n.contains_key(key)).unwrap_or(false)
    }

    pub fn insert_child(&mut self, key: &str) {
        self.children.as_mut().unwrap().insert(key.to_string(), ResContainerNode::new());
    }

    pub fn get_child(&self, key: &str) -> &ResContainerNode {
        self.children.as_ref().unwrap().get(key).unwrap()
    }

    pub fn get_child_mut(&mut self, key: &str) -> &mut ResContainerNode {
        self.children.as_mut().unwrap().get_mut(key).unwrap()
    }

    pub fn get_child_opt(&self, key: &str) -> Option<&ResContainerNode> {
        self.children.as_ref().unwrap().get(key)
    }

    pub fn insert_leaf(&mut self, key: &str, need_crypto_req: bool, need_crypto_resp: bool, need_double_auth: bool) {
        self.children.as_mut().unwrap().insert(
            key.to_string(),
            ResContainerNode {
                children: None,
                leaf_info: Some(ResContainerLeafInfo {
                    need_crypto_req,
                    need_crypto_resp,
                    need_double_auth,
                }),
            },
        );
    }

    pub fn get_leaf_info(&self) -> ResContainerLeafInfo {
        self.leaf_info.as_ref().unwrap().clone()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ResContainerLeafInfo {
    pub need_crypto_req: bool,
    pub need_crypto_resp: bool,
    pub need_double_auth: bool,
}
