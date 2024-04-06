use bios_basic::rbum::rbum_config::RbumConfig;
use bios_sdk_invoke::clients::event_client::EventTopicConfig;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct SearchConfig {
    pub rbum: RbumConfig,
    pub event: Option<EventTopicConfig>,
    pub split_strategy_rule_config: SplitStrategyRuleConfig,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct SplitStrategyRuleConfig {
    pub specify_word_length: Option<usize>,
}
