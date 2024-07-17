use serde::{Deserialize, Serialize};

pub mod event {
    use crate::clients::event_client::{ContextEvent, Event};

    use super::{FlowFrontChangeReq, FlowPostChangeReq};
    pub const FLOW_AVATAR: &str = "flow";
    pub const EVENT_FRONT_CHANGE: &str = "flow/front_change";
    pub const EVENT_POST_CHANGE: &str = "flow/post_change";


    impl Event for FlowFrontChangeReq {
        const CODE: &'static str = EVENT_FRONT_CHANGE;
        fn targets(&self) -> Option<Vec<String>> {
            Some(vec![FLOW_AVATAR.to_string()])
        }
    }
    impl Event for FlowPostChangeReq {
        const CODE: &'static str = EVENT_POST_CHANGE;
        fn targets(&self) -> Option<Vec<String>> {
            Some(vec![FLOW_AVATAR.to_string()])
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlowFrontChangeReq {
    pub inst_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlowPostChangeReq {
    pub inst_id: String,
    pub next_transition_id: String,
}
