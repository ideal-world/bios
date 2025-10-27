use std::collections::HashMap;

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ObjectInitiateMultipartUploadReq {
    pub object_path: String,
    pub content_type: Option<String>,
    pub private: Option<bool>,
    pub special: Option<bool>,
    pub bs_id: Option<String>,
    pub bucket: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ObjectBatchBuildCreatePresignUrlReq {
    pub object_path: String,
    pub upload_id: String,
    pub part_number: u32,
    pub expire_sec: u32,
    pub private: Option<bool>,
    pub special: Option<bool>,
    pub bs_id: Option<String>,
    pub bucket: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ObjectCompleteMultipartUploadReq {
    pub object_path: String,
    pub upload_id: String,
    pub parts: Vec<String>,
    pub private: Option<bool>,
    pub special: Option<bool>,
    pub bs_id: Option<String>,
    pub bucket: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ObjectCopyReq {
    pub from: String,
    pub to: String,
    pub private: Option<bool>,
    pub special: Option<bool>,
    pub bs_id: Option<String>,
    pub bucket: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ObjectPresignBatchViewReq {
    pub object_path: Vec<String>,
    pub expire_sec: u32,
    pub private: Option<bool>,
    pub special: Option<bool>,
    pub obj_exp: Option<u32>,
    pub bs_id: Option<String>,
    pub bucket: Option<String>,
}

impl SpiObjectClient {
    /// Get presigned URL for uploading an object
    ///
    /// 获取上传对象的预签名 URL
    ///
    /// # Arguments
    ///
    /// * `object_path` - Path of the object to upload
    /// * `exp_secs` - Expiration time in seconds for the presigned URL
    /// * `private` - Whether the object is in private bucket
    /// * `special` - Whether the object is in special bucket (for large files)
    /// * `obj_exp` - Object expiration (indicates using temp bucket)
    /// * `bucket` - Custom bucket name (only valid when using custom bs_id)
    /// * `bs_id` - Backend service ID (for custom external services)
    /// * `funs` - Tardis functions instance
    /// * `ctx` - Tardis context
    pub async fn presign_put_obj_url(
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

        let mut url = format!("{object_url}/ci/obj/presign/put?object_path={}&exp_secs={}", object_path, exp_secs);

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
        resp.body.ok_or_else(|| funs.err().not_found("object", "presign_put_url", "Response body is empty", "404-object-presign-empty"))
    }

    /// Get presigned URL for viewing an object
    ///
    /// 获取查看对象的预签名 URL
    ///
    /// # Arguments
    ///
    /// * `object_path` - Path of the object to view
    /// * `exp_secs` - Expiration time in seconds for the presigned URL
    /// * `private` - Whether the object is in private bucket
    /// * `special` - Whether the object is in special bucket (for large files)
    /// * `obj_exp` - Object expiration (indicates using temp bucket)
    /// * `bucket` - Custom bucket name (only valid when using custom bs_id)
    /// * `bs_id` - Backend service ID (for custom external services)
    /// * `funs` - Tardis functions instance
    /// * `ctx` - Tardis context
    pub async fn presign_view_obj_url(
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

        let mut url = format!("{object_url}/ci/obj/presign/view?object_path={}&exp_secs={}", object_path, exp_secs);

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
        resp.body.ok_or_else(|| funs.err().not_found("object", "presign_view_url", "Response body is empty", "404-object-presign-empty"))
    }

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

    /// Batch get presigned URLs for viewing multiple objects
    ///
    /// 批量获取查看多个对象的预签名 URL
    ///
    /// # Arguments
    ///
    /// * `req` - Batch presign view request containing object paths and options
    /// * `funs` - Tardis functions instance
    /// * `ctx` - Tardis context
    ///
    /// # Returns
    ///
    /// HashMap mapping object paths to their presigned URLs
    pub async fn batch_presign_view_obj_url(req: &ObjectPresignBatchViewReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, String>> {
        let object_url = BaseSpiClient::module_url(InvokeModuleKind::Object, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;

        let url = format!("{object_url}/ci/obj/presign/batch_view");

        let resp = funs.web_client().post::<ObjectPresignBatchViewReq, HashMap<String, String>>(&url, req, headers.clone()).await?;
        resp.body.ok_or_else(|| funs.err().not_found("object", "batch_presign_view", "Response body is empty", "404-object-presign-empty"))
    }

    /// Initiate a multipart upload task
    ///
    /// 启动分片上传任务
    ///
    /// # Arguments
    ///
    /// * `req` - Initiate multipart upload request
    /// * `funs` - Tardis functions instance
    /// * `ctx` - Tardis context
    ///
    /// # Returns
    ///
    /// Upload ID for the multipart upload session
    pub async fn initiate_multipart_upload(req: &ObjectInitiateMultipartUploadReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let object_url = BaseSpiClient::module_url(InvokeModuleKind::Object, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;

        let url = format!("{object_url}/ci/obj/multi_upload/initiate_multipart_upload");

        let resp = funs.web_client().post_obj_to_str(&url, req, headers.clone()).await?;
        resp.body.ok_or_else(|| funs.err().not_found("object", "initiate_multipart_upload", "Response body is empty", "404-object-multipart-empty"))
    }

    /// Batch build presigned URLs for uploading parts in multipart upload
    ///
    /// 批量创建分片上传的预签名 URL
    ///
    /// # Arguments
    ///
    /// * `req` - Batch build presign URL request
    /// * `funs` - Tardis functions instance
    /// * `ctx` - Tardis context
    ///
    /// # Returns
    ///
    /// Vector of presigned URLs for uploading parts
    pub async fn batch_build_create_presign_url(req: &ObjectBatchBuildCreatePresignUrlReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        let object_url = BaseSpiClient::module_url(InvokeModuleKind::Object, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;

        let url = format!("{object_url}/ci/obj/multi_upload/batch_build_create_presign_url");

        let resp = funs.web_client().post::<ObjectBatchBuildCreatePresignUrlReq, Vec<String>>(&url, req, headers.clone()).await?;
        resp.body.ok_or_else(|| funs.err().not_found("object", "batch_build_presign_url", "Response body is empty", "404-object-multipart-empty"))
    }

    /// Complete a multipart upload task
    ///
    /// 完成分片上传任务
    ///
    /// # Arguments
    ///
    /// * `req` - Complete multipart upload request containing upload ID and parts
    /// * `funs` - Tardis functions instance
    /// * `ctx` - Tardis context
    pub async fn complete_multipart_upload(req: &ObjectCompleteMultipartUploadReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let object_url = BaseSpiClient::module_url(InvokeModuleKind::Object, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;

        let url = format!("{object_url}/ci/obj/multi_upload/complete_multipart_upload");

        funs.web_client().post_obj_to_str(&url, req, headers.clone()).await?;
        Ok(())
    }

    /// Copy an object from one location to another
    ///
    /// 复制对象从一个位置到另一个位置
    ///
    /// # Arguments
    ///
    /// * `req` - Object copy request containing source and destination paths
    /// * `funs` - Tardis functions instance
    /// * `ctx` - Tardis context
    pub async fn object_copy(req: &ObjectCopyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let object_url = BaseSpiClient::module_url(InvokeModuleKind::Object, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;

        let url = format!("{object_url}/ci/obj/object/copy");

        funs.web_client().post_obj_to_str(&url, req, headers.clone()).await?;
        Ok(())
    }

    /// Check if an object exists
    ///
    /// 检查对象是否存在
    ///
    /// # Arguments
    ///
    /// * `object_path` - Path of the object to check
    /// * `private` - Whether the object is in private bucket
    /// * `special` - Whether the object is in special bucket (for large files)
    /// * `obj_exp` - Object expiration (indicates using temp bucket)
    /// * `bucket` - Custom bucket name (only valid when using custom bs_id)
    /// * `bs_id` - Backend service ID (for custom external services)
    /// * `funs` - Tardis functions instance
    /// * `ctx` - Tardis context
    ///
    /// # Returns
    ///
    /// Boolean indicating whether the object exists
    pub async fn object_exist(
        object_path: &str,
        private: Option<bool>,
        special: Option<bool>,
        obj_exp: Option<u32>,
        bucket: Option<String>,
        bs_id: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<bool> {
        let object_url = BaseSpiClient::module_url(InvokeModuleKind::Object, funs).await?;
        let headers = BaseSpiClient::headers(None, funs, ctx).await?;

        let mut url = format!("{object_url}/ci/obj/object/exist?object_path={}", object_path);

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

        let resp = funs.web_client().get::<bool>(&url, headers.clone()).await?;
        resp.body.ok_or_else(|| funs.err().not_found("object", "object_exist", "Response body is empty", "404-object-exist-empty"))
    }
}
