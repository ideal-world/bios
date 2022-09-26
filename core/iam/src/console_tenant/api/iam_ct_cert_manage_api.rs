use std::collections::HashMap;

use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Path;
use tardis::web::poem_openapi::{param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_cert_dto::{RbumCertDetailResp, RbumCertSummaryWithSkResp};
use bios_basic::rbum::dto::rbum_filer_dto::RbumCertFilterReq;
use bios_basic::rbum::helper::rbum_scope_helper::get_max_level_id_by_context;

use crate::basic::dto::iam_cert_dto::IamCertManageAddReq;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::iam_constants;
use crate::iam_enumeration::IamCertManageKind;

pub struct IamCtCertManageApi;

/// Tenant Console Cert manage API
#[deprecated = "name needs consideration"]
#[poem_openapi::OpenApi(prefix_path = "/ct/cert/manage", tag = "crate::iam_enumeration::Tag::Tenant")]
impl IamCtCertManageApi {
    /// Find Conf
    #[oai(path = "/conf", method = "get")]
    async fn find_conf(&self, ctx: TardisContextExtractor) -> TardisApiResult<HashMap<String, String>> {
        let funs = iam_constants::get_tardis_inst();
        let mut conf_map: HashMap<String, String> = HashMap::new();
        let manage_user_pwd_conf_id =
            IamCertServ::get_cert_conf_id_by_code(IamCertManageKind::ManageUserPwd.to_string().as_str(), get_max_level_id_by_context(&ctx.0), &funs).await?;
        conf_map.insert(IamCertManageKind::ManageUserPwd.to_string(), manage_user_pwd_conf_id);
        let manage_user_visa_conf_id =
            IamCertServ::get_cert_conf_id_by_code(IamCertManageKind::ManageUserVisa.to_string().as_str(), get_max_level_id_by_context(&ctx.0), &funs).await?;
        conf_map.insert(IamCertManageKind::ManageUserVisa.to_string(), manage_user_visa_conf_id);
        TardisResp::ok(conf_map)
    }

    // /// Rest Password
    // #[oai(path = "/rest-sk/", method = "put")]
    // async fn rest_password(
    //     &self,
    //     account_id: Query<String>,
    //     app_id: Query<Option<String>>,
    //     modify_req: Json<IamUserPwdCertRestReq>,
    //     ctx: TardisContextExtractor,
    // ) -> TardisApiResult<Void> {
    //     let ctx = IamCertServ::try_use_app_ctx(ctx.0, app_id.0)?;
    //     let mut funs = iam_constants::get_tardis_inst();
    //     funs.begin().await?;
    //     let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_code(IamCertKernelKind::UserPwd.to_string().as_str(), get_max_level_id_by_context(&ctx), &funs).await?;
    //     RbumCertServ::reset_sk(&cert.id, &modify_req.new_sk.0, &RbumCertFilterReq::default(), funs, ctx).await?;
    //     funs.commit().await?;
    //     TardisResp::ok(Void {})
    // }

    /// Add Manage Cert
    #[oai(path = "/", method = "post")]
    async fn add_manage_cert(&self, add_req: Json<IamCertManageAddReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let id = IamCertServ::add_manage_cert(&add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(id)
    }

    /// modify manage cert ext
    #[oai(path = "/ext/:id", method = "put")]
    async fn modify_manage_cert_ext(&self, id: Path<String>, ext: Query<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertServ::modify_manage_cert_ext(&id.0, &ext.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// get manage cert
    #[oai(path = "/:id", method = "get")]
    async fn get_manage_cert(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<RbumCertSummaryWithSkResp> {
        let funs = iam_constants::get_tardis_inst();
        let cert = IamCertServ::get_manage_cert(&id.0, &funs, &ctx.0).await?;
        TardisResp::ok(cert)
    }

    /// delete manage cert ext
    #[oai(path = "/:id", method = "delete")]
    async fn delete_manage_cert_ext(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertServ::delete_manage_cert(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Paginate Manage Certs
    #[oai(path = "/", method = "get")]
    async fn paginate_certs(
        &self,
        conf_id: Query<Option<String>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<RbumCertDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        let conf_ids = if conf_id.0.is_some() {
            vec![conf_id.0.unwrap()]
        } else {
            let manage_user_pwd_conf_id =
                IamCertServ::get_cert_conf_id_by_code(IamCertManageKind::ManageUserPwd.to_string().as_str(), get_max_level_id_by_context(&ctx.0), &funs).await?;
            let manage_user_visa_conf_id =
                IamCertServ::get_cert_conf_id_by_code(IamCertManageKind::ManageUserVisa.to_string().as_str(), get_max_level_id_by_context(&ctx.0), &funs).await?;
            vec![manage_user_pwd_conf_id, manage_user_visa_conf_id]
        };
        let result = IamCertServ::paginate_certs(
            &RbumCertFilterReq {
                rel_rbum_cert_conf_ids: Some(conf_ids),
                ..Default::default()
            },
            page_number.0,
            page_size.0,
            None,
            None,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(result)
    }

    /// Add Manage rel cert
    #[oai(path = "/:id/rel/:item_id", method = "put")]
    async fn add_rel_item(
        &self,
        id: Path<String>,
        item_id: Path<String>,
        note: Query<Option<String>>,
        ext: Query<Option<String>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Void> {
        let funs = iam_constants::get_tardis_inst();
        IamCertServ::add_rel_cert(&id.0, &item_id.0, note.0, ext.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Manage rel cert
    #[oai(path = "/:id/rel/:item_id", method = "delete")]
    async fn delete_rel_item(&self, id: Path<String>, item_id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = iam_constants::get_tardis_inst();
        IamCertServ::delete_rel_cert(&id.0, &item_id.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Find Manage Certs By item Id
    #[oai(path = "/rel/:item_id", method = "get")]
    async fn find_certs(&self, item_id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<RbumRelBoneResp>> {
        let funs = iam_constants::get_tardis_inst();
        let rbum_certs = IamCertServ::find_simple_rel_cert(&item_id.0, None, None, &funs, &ctx.0).await?;
        TardisResp::ok(rbum_certs)
    }
}
