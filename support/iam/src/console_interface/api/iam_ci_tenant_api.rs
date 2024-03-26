use std::collections::HashMap;

use bios_basic::helper::bios_ctx_helper::unsafe_fill_ctx;
use bios_basic::helper::request_helper::add_remote_ip;
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq, RbumSetCateFilterReq, RbumSetItemRelFilterReq, RbumSetTreeFilterReq};
use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeMainResp;
use bios_basic::rbum::rbum_enumeration::{RbumRelFromKind, RbumSetCateLevelQueryKind};
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::web::poem::web::{Path, Query};
use tardis::web::poem_openapi;
use tardis::web::web_resp::TardisPage;
use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::Request,
    web_resp::{TardisApiResult, TardisResp},
};

use crate::basic::dto::iam_account_dto::{IamAccountDetailAggResp, IamAccountSummaryAggResp};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_enumeration::{IamRelKind, IamSetKind};
use crate::{
    basic::{
        dto::{iam_filer_dto::IamTenantFilterReq, iam_tenant_dto::IamTenantAggDetailResp},
        serv::iam_tenant_serv::IamTenantServ,
    },
    iam_constants,
};

#[derive(Clone, Default)]
pub struct IamCiTenantApi;

#[poem_openapi::OpenApi(prefix_path = "/ci/tenant", tag = "bios_basic::ApiTag::Tenant")]
impl IamCiTenantApi {
    /// Get Current Tenant
    #[oai(path = "/", method = "get")]
    async fn get(&self, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamTenantAggDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        unsafe_fill_ctx(request, &funs, &mut ctx.0).await?;
        add_remote_ip(request, &ctx.0).await?;
        let result = IamTenantServ::get_tenant_agg(&IamTenantServ::get_id_by_ctx(&ctx.0, &funs)?, &IamTenantFilterReq::default(), &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Org Tree By Current Tenant
    ///
    /// * Without parameters: Query the whole tree
    /// * ``parent_sys_code=true`` : query only the next level. This can be used to query level by level when the tree is too large
    #[oai(path = "/orgs", method = "get")]
    async fn get_orgs(
        &self,
        parent_sys_code: Query<Option<String>>,
        set_id: Query<Option<String>>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<RbumSetTreeMainResp>> {
        let funs = iam_constants::get_tardis_inst();
        unsafe_fill_ctx(request, &funs, &mut ctx.0).await?;
        let ctx = IamSetServ::try_get_rel_ctx_by_set_id(set_id.0, &funs, ctx.0).await?;
        add_remote_ip(request, &ctx).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, &ctx).await?;
        let result = IamSetServ::get_tree(
            &set_id,
            &mut RbumSetTreeFilterReq {
                fetch_cate_item: true,
                sys_codes: parent_sys_code.0.map(|parent_sys_code| vec![parent_sys_code]),
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
                sys_code_query_depth: Some(1),
                ..Default::default()
            },
            &funs,
            &ctx,
        )
        .await?;
        ctx.execute_task().await?;
        TardisResp::ok(result.main)
    }
}
