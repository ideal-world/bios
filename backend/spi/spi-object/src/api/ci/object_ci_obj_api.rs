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
///
/// When the built-in service is initialized, 4 buckets are created by default: pub,pri,spe,tamp.
///     pub bucket, we recommend setting it to public read and private write permissions, when is_private passes false, the bucket will be operated.
///     pri bucket, it is recommended to set it to private read-private write, use temporary address to manipulate the object to ensure data security. When is_private passes true, operate the bucket.
///     spe bucket, recommended for large files. When is_special passes true, manipulate this bucket.
///     The tamp bucket, recommended for temporary files, when obj_exp is passed true.
/// 接口控制台对象服务API
/// 内置服务初始化时，默认创建4个桶：pub,pri,spe,tamp
///     pub桶，建议设置为公共读私有写权限，当is_private传false时，操作该桶。
///     pri桶，建议设置为私有读私有写，使用临时地址操作对象，保证数据安全。当is_private传true时，操作该桶。
///     spe桶，建议操作大文件时使用该桶。当is_special传true时，操作该桶。
///     tamp桶，建议操作临时文件时使用该桶，当obj_exp传入时，操作该桶。
#[poem_openapi::OpenApi(prefix_path = "/ci/obj", tag = "bios_basic::ApiTag::Interface")]
impl ObjectCiObjApi {
    /// Fetch URL for temporary authorization of file upload
    ///
    /// 获取用于临时授权上传文件的 URL
    #[oai(path = "/presign/put", method = "get")]
    async fn presign_put_obj_url(
        &self,
        // 对象的路径
        // path of object
        object_path: Query<String>,
        // 临时上传url的生效时长
        // The length of time a temporary upload url is in effect
        exp_secs: Query<u32>,
        // 是否私有
        // private or not
        private: Query<Option<bool>>,
        // 是否特殊
        //Special or not
        special: Query<Option<bool>>,
        // 是否临时，数字表示文件生效时长。
        // 使用obs时，传入数值不生效，仅表示使用tamp桶。
        // Whether or not it is temporary, the number indicates the length of time the file will be in effect.
        // When using obs, passing in a value does not take effect, it only indicates the use of the tamp bucket.
        obj_exp: Query<Option<u32>>,
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
            obj_exp.0,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(url)
    }

    /// Fetch URL for temporary authorization of file delete
    ///
    /// 获取文件删除临时授权的 URL
    #[oai(path = "/presign/delete", method = "get")]
    async fn presign_delete_obj_url(
        &self,
        // 对象的路径
        // path of object
        object_path: Query<String>,
        // 临时上传url的生效时长
        // The length of time a temporary upload url is in effect
        exp_secs: Query<u32>,
        // 是否私有
        // private or not
        private: Query<Option<bool>>,
        // 是否特殊
        //Special or not
        special: Query<Option<bool>>,
        // 是否临时，数字表示文件生效时长。
        // 使用obs时，传入数值不生效，仅表示使用tamp桶。
        // Whether or not it is temporary, the number indicates the length of time the file will be in effect.
        // When using obs, passing in a value does not take effect, it only indicates the use of the tamp bucket.
        obj_exp: Query<Option<u32>>,
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
            obj_exp.0,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(url)
    }

    /// Fetch URL for temporary authorization of file
    ///
    /// 获取文件临时授权的 URL
    #[oai(path = "/presign/view", method = "get")]
    async fn presign_view_obj_url(
        &self,
        // 对象的路径
        // path of object
        object_path: Query<String>,
        // 临时上传url的生效时长
        // The length of time a temporary upload url is in effect
        exp_secs: Query<u32>,
        // 是否私有
        // private or not
        private: Query<Option<bool>>,
        // 是否特殊
        //Special or not
        special: Query<Option<bool>>,
        // 是否临时，数字表示文件生效时长。
        // 使用obs时，传入数值不生效，仅表示使用tamp桶。
        // Whether or not it is temporary, the number indicates the length of time the file will be in effect.
        // When using obs, passing in a value does not take effect, it only indicates the use of the tamp bucket.
        obj_exp: Query<Option<u32>>,
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
            obj_exp.0,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(url)
    }

    /// Batch fetch URL for temporary authorization of file
    ///
    /// 批量获取文件临时授权的 URL
    #[oai(path = "/presign/batch_view", method = "post")]
    async fn batch_presign_view_obj_url(&self, req: Json<ObjectPresignBatchViewReq>, ctx: TardisContextExtractor) -> TardisApiResult<HashMap<String, String>> {
        let funs = crate::get_tardis_inst();
        let url = object_obj_serv::batch_get_presign_obj_url(req.0.object_path, req.0.expire_sec, req.0.private, req.0.special, req.0.obj_exp, &funs, &ctx.0).await?;
        TardisResp::ok(url)
    }

    /// Multipart Upload:Initiate a Multipart Upload Task
    ///
    /// 分片上传： 启动分片上传任务
    #[oai(path = "/multi_upload/initiate_multipart_upload", method = "post")]
    async fn initiate_multipart_upload(&self, req: Json<ObjectInitiateMultipartUploadReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let funs = crate::get_tardis_inst();
        let upload_id = object_obj_serv::initiate_multipart_upload(req.0, &funs, &ctx.0).await?;
        TardisResp::ok(upload_id)
    }

    /// Multipart Upload:Create pre-signed URLs for each part
    ///
    /// 分片上传： 为每个部分创建预签名 URL
    #[oai(path = "/multi_upload/batch_build_create_presign_url", method = "post")]
    async fn batch_build_create_presign_url(&self, req: Json<ObjectBatchBuildCreatePresignUrlReq>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<String>> {
        let funs = crate::get_tardis_inst();
        let presign_urls = object_obj_serv::batch_build_create_presign_url(req.0, &funs, &ctx.0).await?;
        TardisResp::ok(presign_urls)
    }

    /// Multipart Upload:Complete Multipart Upload Task
    ///
    /// 分片上传： 完成分片上传任务
    #[oai(path = "/multi_upload/complete_multipart_upload", method = "post")]
    async fn complete_multipart_upload(&self, req: Json<ObjectCompleteMultipartUploadReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        object_obj_serv::complete_multipart_upload(req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void)
    }

    /// Create A Copy Of An Object That Is Already Stored
    ///
    /// 创建已存储对象的副本
    #[oai(path = "/object/copy", method = "post")]
    async fn object_copy(&self, req: Json<ObjectCopyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        object_obj_serv::object_copy(req.0.from, req.0.to, req.0.private, req.0.special, &funs, &ctx.0).await?;
        TardisResp::ok(Void)
    }

    /// Deleting A Single Object
    ///
    /// 删除单个对象
    #[oai(path = "/object", method = "delete")]
    async fn object_delete(
        &self,
        // 对象的路径
        // path of object
        object_path: Query<String>,
        // 是否私有
        // private or not
        private: Query<Option<bool>>,
        // 是否特殊
        //Special or not
        special: Query<Option<bool>>,
        // 是否临时，数字表示文件生效时长。
        // 使用obs时，传入数值不生效，仅表示使用tamp桶。
        // Whether or not it is temporary, the number indicates the length of time the file will be in effect.
        // When using obs, passing in a value does not take effect, it only indicates the use of the tamp bucket.
        obj_exp: Query<Option<u32>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Void> {
        let funs = crate::get_tardis_inst();
        object_obj_serv::object_delete(object_path.0, private.0, special.0, obj_exp.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void)
    }

    /// Batch deleting Objects
    ///
    /// 批量删除对象
    #[oai(path = "/object/batch_delete", method = "delete")]
    async fn batch_object_delete(&self, req: Json<ObjectBatchDeleteReq>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<String>> {
        let funs = crate::get_tardis_inst();
        TardisResp::ok(object_obj_serv::batch_object_delete(req.0.object_path, req.0.private, req.0.special, req.0.obj_exp, &funs, &ctx.0).await?)
    }

    /// Check object is exist
    ///
    /// 检查对象是否存在
    #[oai(path = "/object/exist", method = "get")]
    async fn object_exist(
        &self,
        // 对象的路径
        // path of object
        object_path: Query<String>,
        // 是否私有
        // private or not
        private: Query<Option<bool>>,
        // 是否特殊
        //Special or not
        special: Query<Option<bool>>,
        // 是否临时，数字表示文件生效时长。
        // 使用obs时，传入数值不生效，仅表示使用tamp桶。
        // Whether or not it is temporary, the number indicates the length of time the file will be in effect.
        // When using obs, passing in a value does not take effect, it only indicates the use of the tamp bucket.
        obj_exp: Query<Option<u32>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<bool> {
        let funs = crate::get_tardis_inst();
        TardisResp::ok(object_obj_serv::object_exist(object_path.0, private.0, special.0, obj_exp.0, &funs, &ctx.0).await?)
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
