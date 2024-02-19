#[derive(Clone)]
pub struct CertInfo {
    pub id: String,
    pub name: Option<String>,
    pub roles: Vec<RoleInfo>,
}

#[derive(Clone)]
pub struct RoleInfo {
    pub id: String,
    pub name: Option<String>,
}
