use serde::{Deserialize, Serialize};

pub mod event {
    use crate::clients::event_client::{ContextEvent, Event};

    use super::{FlowFrontChangeReq, FlowPostChangeReq};

    pub const EVENT_FRONT_CHANGE: &str = "event/front_change";
    pub const EVENT_POST_CHANGE: &str = "event/post_change";
    pub type FlowFrontChangeEvent = ContextEvent<FlowFrontChangeReq>;
    pub type FlowPostChangeEvent = ContextEvent<FlowPostChangeReq>;

    impl Event for FlowFrontChangeEvent {
        const CODE: &'static str = EVENT_FRONT_CHANGE;
    }
    impl Event for FlowPostChangeEvent {
        const CODE: &'static str = EVENT_POST_CHANGE;
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
