use std::collections::HashMap;

use tardis::web::context_extractor::TardisContextExtractor;

use tardis::web::poem::web::Json;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::dto::object_dto::{
    ObjectBatchBuildCreatePresignUrlReq, ObjectBatchDeleteReq, ObjectCompleteMultipartUploadReq, ObjectCopyReq, ObjectInitiateMultipartUploadReq, ObjectObjPresignKind,
    ObjectPresignBatchViewReq,
};
use crate::serv::object_obj_serv;
#[derive(Clone)]
pub struct ObjectCiObjApi;

/// Interface Console Object API
#[poem_openapi::OpenApi(prefix_path = "/ci/obj", tag = "bios_basic::ApiTag::Interface")]
impl ObjectCiObjApi {
    /// Fetch URL for temporary authorization of file upload
    #[oai(path = "/presign/put", method = "get")]
    async fn presign_put_obj_url(
        &self,
        object_path: Query<String>,
        exp_secs: Query<u32>,
        private: Query<Option<bool>>,
        special: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<String> {
        let funs = crate::get_tardis_inst();
        let url = object_obj_serv::presign_obj_url(
            ObjectObjPresignKind::Upload,
            object_path.0.trim(),
            None,
            None,
            exp_secs.0,
            private.0,
            special.0,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(url)
    }

    /// Fetch URL for temporary authorization of file delete
    #[oai(path = "/presign/delete", method = "get")]
    async fn presign_delete_obj_url(
        &self,
        object_path: Query<String>,
        exp_secs: Query<u32>,
        private: Query<Option<bool>>,
        special: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<String> {
        let funs = crate::get_tardis_inst();
        let url = object_obj_serv::presign_obj_url(
            ObjectObjPresignKind::Delete,
            object_path.0.trim(),
            None,
            None,
            exp_secs.0,
            private.0,
            special.0,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(url)
    }

    /// Fetch URL for temporary authorization of file
    #[oai(path = "/presign/view", method = "get")]
    async fn presign_view_obj_url(
        &self,
        object_path: Query<String>,
        exp_secs: Query<u32>,
        private: Query<Option<bool>>,
        special: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<String> {
        let funs = crate::get_tardis_inst();
        let url = object_obj_serv::presign_obj_url(
            ObjectObjPresignKind::View,
            object_path.0.trim(),
            None,
            None,
            exp_secs.0,
            private.0,
            special.0,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(url)
    }

    /// Batch fetch URL for temporary authorization of file
    #[oai(path = "/presign/batch_view", method = "get")]
    async fn batch_presign_view_obj_url(&self, req: Json<ObjectPresignBatchViewReq>, ctx: TardisContextExtractor) -> TardisApiResult<HashMap<String, String>> {
        let funs = crate::get_tardis_inst();
        let url = object_obj_serv::batch_get_presign_obj_url(req.0.object_path, req.0.expire_sec, req.0.private, req.0.special, &funs, &ctx.0).await?;
        TardisResp::ok(url)
    }

    /// Initiate a Multipart Upload Task
    #[oai(path = "/multi_upload/initiate_multipart_upload", method = "post")]
    async fn initiate_multipart_upload(&self, req: Json<ObjectInitiateMultipartUploadReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let funs = crate::get_tardis_inst();
        let upload_id = object_obj_serv::initiate_multipart_upload(req.0, &funs, &ctx.0).await?;
        TardisResp::ok(upload_id)
    }

    /// Create pre-signed URLs for each part
    #[oai(path = "/multi_upload/batch_build_create_presign_url", method = "post")]
    async fn batch_build_create_presign_url(&self, req: Json<ObjectBatchBuildCreatePresignUrlReq>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<String>> {
        let funs = crate::get_tardis_inst();
        let presign_urls = object_obj_serv::batch_build_create_presign_url(req.0, &funs, &ctx.0).await?;
        TardisResp::ok(presign_urls)
    }

    /// Complete Multipart Upload Task
    #[oai(path = "/multi_upload/batch_build_create_presign_url", method = "post")]
    async fn complete_multipart_upload(&self, req: Json<ObjectCompleteMultipartUploadReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        object_obj_serv::complete_multipart_upload(req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void)
    }

    /// Create A Copy Of An Object That Is Already Stored
    #[oai(path = "/object/copy", method = "post")]
    async fn object_copy(&self, req: Json<ObjectCopyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        object_obj_serv::object_copy(req.0.from, req.0.to, req.0.private, req.0.special, &funs, &ctx.0).await?;
        TardisResp::ok(Void)
    }

    /// Deleting A Single Object
    #[oai(path = "/object", method = "delete")]
    async fn object_delete(&self, object_path: Query<String>, private: Query<Option<bool>>, special: Query<Option<bool>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        object_obj_serv::object_delete(object_path.0, private.0, special.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void)
    }

    /// Deleting Objects
    #[oai(path = "/object/batch_delete", method = "delete")]
    async fn batch_object_delete(&self, req: Json<ObjectBatchDeleteReq>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<String>> {
        let funs = crate::get_tardis_inst();
        TardisResp::ok(object_obj_serv::batch_object_delete(req.0.object_path, req.0.private, req.0.special, &funs, &ctx.0).await?)
    }

    /// Check object is exist
    #[oai(path = "/object/exist", method = "get")]
    async fn object_exist(&self, object_path: Query<String>, private: Query<Option<bool>>, special: Query<Option<bool>>, ctx: TardisContextExtractor) -> TardisApiResult<bool> {
        let funs = crate::get_tardis_inst();
        TardisResp::ok(object_obj_serv::object_exist(object_path.0, private.0, special.0, &funs, &ctx.0).await?)
    }

    // /// Fetch URL for temporary authorization of thumbnail
    // #[oai(path = "/presign/thumbnail", method = "get")]
    // async fn presign_get_thumbnail_url(
    //     &self,
    //     object_path: Query<String>,
    //     max_width: Query<Option<String>>,
    //     max_height: Query<Option<String>>,
    //     exp_secs: Query<u32>,
    //     private: Query<bool>,
    //     ctx: TardisContextExtractor,
    //
    // ) -> TardisApiResult<String> {
    //     let funs = crate::get_tardis_inst();
    //     let url = object_obj_serv::presign_obj_url(
    //         ObjectObjPresignKind::View,
    //         object_path.0.trim(),
    //         max_width.0,
    //         max_height.0,
    //         exp_secs.0,
    //         private.0,
    //         &funs,
    //         &ctx.0,
    //     )
    //     .await?;
    //     TardisResp::ok(url)
    // }
}
