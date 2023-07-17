mod enums;
pub use enums::*;
mod message;
pub use message::*;
mod trigger;
pub use trigger::*;
mod content_replace;
pub use content_replace::*;
mod vcode_strategy;
pub use vcode_strategy::*;

use tardis::web::web_resp::Void;
pub const VOID: Void = Void {};
