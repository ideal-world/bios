mod reach_api_cc_message;
pub use reach_api_cc_message::ReachMessageCcApi;
mod reach_api_cc_trigger_scene;
pub use reach_api_cc_trigger_scene::ReachTriggerSceneCcApi;

pub type ReachCcApi = (ReachTriggerSceneCcApi, ReachMessageCcApi);
