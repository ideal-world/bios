use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct CertInfo {
    pub id: String,
    pub own_paths: Option<String>,
    pub name: Option<String>,
    pub roles: Vec<RoleInfo>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RoleInfo {
    pub id: String,
    pub name: Option<String>,
}
