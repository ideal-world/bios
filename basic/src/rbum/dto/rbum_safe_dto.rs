use tardis::{
    chrono::{DateTime, Utc},
    web::poem_openapi,
};

#[derive(Debug, poem_openapi::Object, Clone, Default)]
pub struct RbumSafeSummaryResp {
    pub id: String,
    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

impl RbumSafeSummaryResp {
    pub fn extends_to_detail_resp(self, owner_name: impl Into<String>) -> RbumSafeDetailResp {
        RbumSafeDetailResp {
            id: self.id,
            own_paths: self.own_paths,
            owner: self.owner,
            owner_name: owner_name.into(),
            create_time: self.create_time,
            update_time: self.update_time,
        }
    }
}

#[derive(Debug, poem_openapi::Object, Clone, Default)]
pub struct RbumSafeDetailResp {
    pub id: String,
    pub own_paths: String,
    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
