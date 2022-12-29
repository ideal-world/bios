use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ObjectObjPresignKind {
    Upload,
    Delete,
    View,
}
