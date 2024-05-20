use bios_sdk_invoke::clients::spi_log_client::{LogDynamicContentReq, SpiLogClient};
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Path;
use tardis::web::poem_openapi::{param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_cert_dto::{RbumCertDetailResp, RbumCertSummaryWithSkResp};
use bios_basic::rbum::dto::rbum_filer_dto::RbumCertFilterReq;
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;

use crate::basic::dto::iam_cert_dto::{IamCertManageAddReq, IamCertManageModifyReq};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::iam_constants;
use crate::iam_enumeration::IamCertExtKind;
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCtCertManageApi;

/// Tenant Console Cert manage API
/// 租户控制台证书管理API
#[poem_openapi::OpenApi(prefix_path = "/ct/cert/manage", tag = "bios_basic::ApiTag::Tenant")]
impl IamCtCertManageApi {
    /// Add Manage Cert
    /// 添加凭证
    #[oai(path = "/", method = "post")]
    async fn add_manage_cert(&self, add_req: Json<IamCertManageAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let id = IamCertServ::add_manage_cert(&add_req.0, &funs, &ctx.0.clone()).await?;
        let _ = SpiLogClient::add_dynamic_log(
            &LogDynamicContentReq {
                details: None,
                sub_kind: None,
                content: Some(format!("凭证 {}", add_req.0.ak)),
            },
            None,
            Some("dynamic_log_cert_manage".to_string()),
            Some(id.clone()),
            Some("新增".to_string()),
            None,
            Some(tardis::chrono::Utc::now().to_rfc3339()),
            &funs,
            &ctx.0,
        )
        .await;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(id)
    }

    /// modify manage cert
    /// 修改凭证
    #[oai(path = "/:id", method = "put")]
    async fn modify_manage_cert(&self, id: Path<String>, modify_req: Json<IamCertManageModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertServ::modify_manage_cert(&id.0, &modify_req.0, &funs, &ctx.0).await?;
        let _ = SpiLogClient::add_dynamic_log(
            &LogDynamicContentReq {
                details: None,
                sub_kind: None,
                content: Some(format!("凭证 {}", modify_req.0.ak)),
            },
            None,
            Some("dynamic_log_cert_manage".to_string()),
            Some(id.0.to_string()),
            Some("编辑".to_string()),
            None,
            Some(tardis::chrono::Utc::now().to_rfc3339()),
            &funs,
            &ctx.0,
        )
        .await;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// modify manage cert ext
    /// 修改凭证扩展信息
    #[oai(path = "/ext/:id", method = "put")]
    async fn modify_manage_cert_ext(&self, id: Path<String>, ext: Query<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertServ::modify_manage_cert_ext(&id.0, &ext.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// get manage cert
    /// 获取凭证
    #[oai(path = "/:id", method = "get")]
    async fn get_manage_cert(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<RbumCertSummaryWithSkResp> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let cert = IamCertServ::get_3th_kind_cert_by_id(&id.0, true, &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(cert)
    }

    /// delete manage cert ext
    /// 删除凭证
    #[oai(path = "/:id", method = "delete")]
    async fn delete_manage_cert_ext(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let cert = IamCertServ::get_3th_kind_cert_by_id(&id.0, false, &funs, &ctx.0).await?;
        IamCertServ::delete_manage_cert(&id.0, &funs, &ctx.0).await?;
        let _ = SpiLogClient::add_dynamic_log(
            &LogDynamicContentReq {
                details: None,
                sub_kind: None,
                content: Some(format!("凭证 {}", cert.ak)),
            },
            None,
            Some("dynamic_log_cert_manage".to_string()),
            Some(id.clone()),
            Some("删除".to_string()),
            None,
            Some(tardis::chrono::Utc::now().to_rfc3339()),
            &funs,
            &ctx.0,
        )
        .await;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Paginate Manage Certs for tenant
    /// 分页查询凭证
    #[oai(path = "/", method = "get")]
    async fn paginate_certs(
        &self,
        page_number: Query<u32>,
        page_size: Query<u32>,
        supplier: Query<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<RbumCertDetailResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamCertServ::paginate_certs(
            &RbumCertFilterReq {
                kind: Some(IamCertExtKind::ThirdParty.to_string()),
                suppliers: Some(supplier.0.split(',').map(|str| str.to_string()).collect()),
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
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Add Manage rel cert
    /// 添加凭证关联
    #[oai(path = "/:id/rel/:item_id", method = "put")]
    async fn add_rel_item(
        &self,
        id: Path<String>,
        item_id: Path<String>,
        note: Query<Option<String>>,
        ext: Query<Option<String>>,
        own_paths: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertServ::add_rel_cert(&id.0, &item_id.0, note.0, ext.0, own_paths.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Manage rel cert
    /// 删除凭证关联
    #[oai(path = "/:id/rel/:item_id", method = "delete")]
    async fn delete_rel_item(&self, id: Path<String>, item_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertServ::delete_rel_cert(&id.0, &item_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Find Manage Certs By item Id
    /// 根据关联ID获取凭证
    #[oai(path = "/rel/:item_id", method = "get")]
    async fn find_certs(&self, item_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<RbumRelBoneResp>> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx).await?;
        let rbum_certs = IamCertServ::find_to_simple_rel_cert(&item_id.0, None, None, &funs, &ctx).await?;
        ctx.execute_task().await?;
        TardisResp::ok(rbum_certs)
    }
}
