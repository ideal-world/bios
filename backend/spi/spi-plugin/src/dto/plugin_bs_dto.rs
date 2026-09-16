use bios_basic::rbum::dto::rbum_rel_agg_dto::{RbumRelAggResp, RbumRelAttrAggAddReq};
use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct PluginBsAddReq {
    pub rel_id: Option<String>,
    pub bs_id: String,
    pub app_tenant_id: String,
    pub name: String,
    pub attrs: Option<Vec<RbumRelAttrAggAddReq>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct PluginBsInfoResp {
    pub id: String,
    pub name: String,
    pub kind_id: String,
    pub kind_code: String,
    pub kind_name: String,
    pub kind_parent_id: Option<String>,
    pub kind_parent_name: Option<String>,
    pub rel: Option<RbumRelAggResp>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct PluginBsCertInfoResp {
    pub id: String,
    pub name: String,
    pub conn_uri: String,
    pub ak: String,
    pub sk: String,
    pub ext: String,
    pub private: bool,
    pub rel: Option<RbumRelAggResp>,
}

/// The result of encrypting the sensitive relationship attributes which are stored in plaintext
///
/// 明文存储的插件关联关系敏感属性的加密结果
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Object))]
pub struct PluginRelSecretMigrateResp {
    /// Is it a dry run
    ///
    /// 是否仅预演
    pub dry_run: bool,
    /// The total number of the sensitive attributes which are stored in plaintext
    ///
    /// 明文存储的敏感属性总数
    pub total_size: u64,
    /// The number of the records that have been encrypted (always 0 in the dry run)
    ///
    /// 已加密的记录数（预演时恒为 0）
    pub processed_size: u64,
    /// The number of the records that failed to be converted
    ///
    /// 转换失败的记录数
    pub failed_size: u64,
    /// The id samples of the records to be encrypted
    ///
    /// 待加密记录的id样例
    pub sample_ids: Vec<String>,
}
