pub struct CertInfo {
    pub id: String,
    pub name: Option<String>,
    pub roles: Vec<RoleInfo>,
}

pub struct RoleInfo {
    pub id: String,
    pub name: Option<String>,
}
