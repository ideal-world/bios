use tardis::web::context_extractor::TardisContextExtractor;

use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::dto::object_dto::ObjectObjPresignKind;
use crate::serv::object_obj_serv;
#[derive(Clone)]
pub struct ObjectCiObjApi;

/// Interface Console Object API
#[poem_openapi::OpenApi(prefix_path = "/ci/obj", tag = "bios_basic::ApiTag::Interface")]
impl ObjectCiObjApi {
    /// Fetch URL for temporary authorization of file upload
    #[oai(path = "/presign/put", method = "get")]
    async fn presign_put_obj_url(&self, object_path: Query<String>, exp_secs: Query<u32>, private: Query<bool>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let funs = crate::get_tardis_inst();
        let url = object_obj_serv::presign_obj_url(ObjectObjPresignKind::Upload, object_path.0.trim(), None, None, exp_secs.0, private.0, &funs, &ctx.0).await?;
        TardisResp::ok(url)
    }

    /// Fetch URL for temporary authorization of file delete
    #[oai(path = "/presign/delete", method = "get")]
    async fn presign_delete_obj_url(&self, object_path: Query<String>, exp_secs: Query<u32>, private: Query<bool>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let funs = crate::get_tardis_inst();
        let url = object_obj_serv::presign_obj_url(ObjectObjPresignKind::Delete, object_path.0.trim(), None, None, exp_secs.0, private.0, &funs, &ctx.0).await?;
        TardisResp::ok(url)
    }

    /// Fetch URL for temporary authorization of file
    #[oai(path = "/presign/view", method = "get")]
    async fn presign_view_obj_url(&self, object_path: Query<String>, exp_secs: Query<u32>, private: Query<bool>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let funs = crate::get_tardis_inst();
        let url = object_obj_serv::presign_obj_url(ObjectObjPresignKind::View, object_path.0.trim(), None, None, exp_secs.0, private.0, &funs, &ctx.0).await?;
        TardisResp::ok(url)
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
