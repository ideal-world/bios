use serde::{Deserialize, Serialize};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use crate::invoke_enumeration::InvokeModuleKind;

use super::base_spi_client::BaseSpiClient;

#[derive(Clone, Debug, Default)]
pub struct SpiObjectClient;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ObjectBatchDeleteReq {
    pub object_path: Vec<String>,
    pub private: Option<bool>,
    pub special: Option<bool>,
    pub obj_exp: Option<u32>,
    pub bs_id: Option<String>,
    pub bucket: Option<String>,
}

impl SpiObjectClient {
    /// Get presigned URL for deleting an object
    ///
    /// 获取删除对象的预签名 URL
    ///
    /// # Arguments
    ///
    /// * `object_path` - Path of the object to delete
    /// * `exp_secs` - Expiration time in seconds for the presigned URL
    /// * `private` - Whether the object is in private bucket
    /// * `special` - Whether the object is in special bucket (for large files)
    /// * `obj_exp` - Object expiration (indicates using temp bucket)
    /// * `bucket` - Custom bucket name (only valid when using custom bs_id)
    /// * `bs_id` - Backend service ID (for custom external services)
    /// * `funs` - Tardis functions instance
    /// * `ctx` - Tardis context
    pub async fn presign_delete_obj_url(
        object_path: &str,
        exp_secs: u32,
        private: Option<bool>,
        special: Option<bool>,
        obj_exp: Option<u32>,
        bucket: Option<String>,
        bs_id: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        let object_url = BaseSpiClient::module_url(InvokeModuleKind::Object, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;

        let mut url = format!("{object_url}/ci/obj/presign/delete?object_path={}&exp_secs={}", object_path, exp_secs);

        if let Some(private) = private {
            url = format!("{url}&private={private}");
        }
        if let Some(special) = special {
            url = format!("{url}&special={special}");
        }
        if let Some(obj_exp) = obj_exp {
            url = format!("{url}&obj_exp={obj_exp}");
        }
        if let Some(bucket) = bucket {
            url = format!("{url}&bucket={bucket}");
        }
        if let Some(bs_id) = bs_id {
            url = format!("{url}&bs_id={bs_id}");
        }

        let resp = funs.web_client().get_to_str(&url, headers.clone()).await?;
        resp.body.ok_or_else(|| funs.err().not_found("object", "presign_delete_url", "Response body is empty", "404-object-presign-empty"))
    }

    /// Delete a single object
    ///
    /// 删除单个对象
    ///
    /// # Arguments
    ///
    /// * `object_path` - Path of the object to delete
    /// * `private` - Whether the object is in private bucket
    /// * `special` - Whether the object is in special bucket (for large files)
    /// * `obj_exp` - Object expiration (indicates using temp bucket)
    /// * `bucket` - Custom bucket name (only valid when using custom bs_id)
    /// * `bs_id` - Backend service ID (for custom external services)
    /// * `funs` - Tardis functions instance
    /// * `ctx` - Tardis context
    pub async fn delete_object(
        object_path: &str,
        private: Option<bool>,
        special: Option<bool>,
        obj_exp: Option<u32>,
        bucket: Option<String>,
        bs_id: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let object_url = BaseSpiClient::module_url(InvokeModuleKind::Object, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;

        let mut url = format!("{object_url}/ci/obj/object?object_path={}", object_path);

        if let Some(private) = private {
            url = format!("{url}&private={private}");
        }
        if let Some(special) = special {
            url = format!("{url}&special={special}");
        }
        if let Some(obj_exp) = obj_exp {
            url = format!("{url}&obj_exp={obj_exp}");
        }
        if let Some(bucket) = bucket {
            url = format!("{url}&bucket={bucket}");
        }
        if let Some(bs_id) = bs_id {
            url = format!("{url}&bs_id={bs_id}");
        }

        funs.web_client().delete_to_void(&url, headers.clone()).await?;
        Ok(())
    }

    /// Batch delete multiple objects
    ///
    /// 批量删除多个对象
    ///
    /// # Arguments
    ///
    /// * `req` - Batch delete request containing object paths and options
    /// * `funs` - Tardis functions instance
    /// * `ctx` - Tardis context
    ///
    /// # Returns
    ///
    /// Vector of object paths that failed to delete (empty if all succeeded)
    pub async fn batch_delete_objects(req: &ObjectBatchDeleteReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        let object_url = BaseSpiClient::module_url(InvokeModuleKind::Object, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;

        let url = format!("{object_url}/ci/obj/object/batch_delete");

        let resp = funs.web_client().delete_with_body::<Vec<String>, ObjectBatchDeleteReq>(&url, headers.clone(), req).await?;
        resp.body.ok_or_else(|| funs.err().not_found("object", "batch_delete", "Response body is empty", "404-object-batch-delete-empty"))
    }
}
