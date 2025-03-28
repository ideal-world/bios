use serde::{Deserialize, Serialize, Serializer};

use tardis::basic::field::TrimString;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::{self, DbErr, FromQueryResult, QueryResult};
use tardis::web::poem_openapi;

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct IamSubDeployLicenseAddReq {
    pub name: Option<TrimString>,
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub sub_deploy_id: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct IamSubDeployLicenseModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    pub name: TrimString,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct IamSubDeployLicenseDetailResp {
    pub id: String,
    pub name: String,
    pub sub_deploy_id: String,
    pub license: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,

    pub own_paths: String,
    pub owner: String,
    pub owner_name: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub create_by: String,
    pub update_by: String,
}

impl FromQueryResult for IamSubDeployLicenseDetailResp {
    fn from_query_result(res: &QueryResult, pre: &str) -> Result<Self, DbErr> {
        // 先正常获取所有字段
        let id: String = res.try_get(pre, "id")?;
        let name: String = res.try_get(pre, "name")?;
        let sub_deploy_id: String = res.try_get(pre, "sub_deploy_id")?;
        let license: String = res.try_get(pre, "license")?;
        let start_time: DateTime<Utc> = res.try_get(pre, "start_time")?;
        let end_time: DateTime<Utc> = res.try_get(pre, "end_time")?;
        let own_paths: String = res.try_get(pre, "own_paths")?;
        let owner: String = res.try_get(pre, "owner")?;
        let owner_name: Option<String> = res.try_get(pre, "owner_name").ok();
        let create_time: DateTime<Utc> = res.try_get(pre, "create_time")?;
        let update_time: DateTime<Utc> = res.try_get(pre, "update_time")?;
        let create_by: String = res.try_get(pre, "create_by")?;
        let update_by: String = res.try_get(pre, "update_by")?;

        // 在从数据库加载时就对 license 进行脱敏处理
        let license_len = license.len();
        let masked_license = if license_len <= 8 {
            if license_len <= 2 {
                license.clone()
            } else {
                let first = &license[0..1];
                let last = &license[license_len - 1..];
                let stars = "*".repeat(license_len - 2);
                format!("{}{}{}", first, stars, last)
            }
        } else {
            let prefix = &license[0..4];
            let suffix = &license[license_len - 4..];
            let stars = "*".repeat(license_len - 8);
            format!("{}{}{}", prefix, stars, suffix)
        };

        Ok(Self {
            id,
            name,
            sub_deploy_id,
            license: masked_license, // 使用脱敏后的 license
            start_time,
            end_time,
            own_paths,
            owner,
            owner_name,
            create_time,
            update_time,
            create_by,
            update_by,
        })
    }
}
