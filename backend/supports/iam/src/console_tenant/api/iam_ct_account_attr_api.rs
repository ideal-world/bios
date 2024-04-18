use std::collections::HashMap;

use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::dto::rbum_kind_attr_dto::{RbumKindAttrDetailResp, RbumKindAttrModifyReq, RbumKindAttrSummaryResp};

use crate::basic::dto::iam_attr_dto::IamKindAttrAddReq;
use crate::basic::serv::iam_attr_serv::IamAttrServ;
use crate::iam_constants;
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCtAccountAttrApi;

/// Tenant Console Account Attr API
/// 租户控制台账号属性API
///
/// Note: the current account attr only supports tenant level.
/// 注意：当前账号属性仅支持租户级别。
#[poem_openapi::OpenApi(prefix_path = "/ct/account/attr", tag = "bios_basic::ApiTag::Tenant")]
impl IamCtAccountAttrApi {
    /// Add Account Attr
    /// 添加账号属性
    #[oai(path = "/", method = "post")]
    async fn add_attr(&self, add_req: Json<IamKindAttrAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamAttrServ::add_account_attr(&add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Account Attr By Account Attr Id
    /// 修改账号属性
    #[oai(path = "/:id", method = "put")]
    async fn modify_attr(&self, id: Path<String>, mut modify_req: Json<RbumKindAttrModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAttrServ::modify_account_attr(&id.0, &mut modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Get Account Attr By Account Attr Id
    /// 根据账号属性ID获取账号属性
    #[oai(path = "/:id", method = "get")]
    async fn get_attr(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<RbumKindAttrDetailResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamAttrServ::get_account_attr(&id.0, true, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Account Attrs
    /// 查找账号属性
    #[oai(path = "/", method = "get")]
    async fn find_attrs(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<RbumKindAttrSummaryResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamAttrServ::find_account_attrs(&funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Delete Account Attr By Account Attr Id
    /// 删除账号属性
    #[oai(path = "/:id", method = "delete")]
    async fn delete_attr(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAttrServ::delete_account_attr(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Find Account Ext Attr Values By Account Id
    /// 查找账号扩展属性值
    #[oai(path = "/value", method = "get")]
    async fn find_account_attr_values(&self, account_id: Query<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<HashMap<String, String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamAttrServ::find_account_attr_values(&account_id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
