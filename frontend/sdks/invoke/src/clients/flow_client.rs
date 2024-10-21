use serde::{Deserialize, Serialize};

#[cfg(feature = "event")]
pub mod event {
    use asteroid_mq::prelude::*;

    use super::{FlowFrontChangeReq, FlowPostChangeReq};
    pub const FLOW_AVATAR: &str = "flow";
    pub const EVENT_FRONT_CHANGE: &str = "flow/front_change";
    pub const EVENT_POST_CHANGE: &str = "flow/post_change";

    impl EventAttribute for FlowFrontChangeReq {
        const SUBJECT: Subject = Subject::const_new("flow/front_change");
    }
    impl EventAttribute for FlowPostChangeReq {
        const SUBJECT: Subject = Subject::const_new("flow/post_change");
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
