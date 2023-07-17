mod message;
pub use message::ReachMessageCcApi;
mod trigger_scene;
pub use trigger_scene::ReachTriggerSceneCcApi;

pub type ReachCcApi = (ReachTriggerSceneCcApi, ReachMessageCcApi);
