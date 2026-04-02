use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use itertools::Itertools;
use tardis::basic::error::TardisError;
use tardis::futures::future::join_all;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::basic::dto::iam_filer_dto::IamThirdPartyAppFilterReq;
use crate::iam_enumeration::IamThirdPartyAppStatusKind;
use crate::basic::dto::iam_third_party_app_dto::{
    IamThirdPartyAppAddReq, IamThirdPartyAppDetailResp, IamThirdPartyAppModifyReq, IamThirdPartyAppSummaryResp,
};
use crate::basic::serv::iam_third_party_app_serv::IamThirdPartyAppServ;
use crate::iam_constants;

#[derive(Clone, Default)]
pub struct IamCcThirdPartyAppApi;

/// Common Console Third Party App API
/// 通用控制台第三方应用API
#[poem_openapi::OpenApi(prefix_path = "/cc/third_party_app", tag = "bios_basic::ApiTag::Common")]
impl IamCcThirdPartyAppApi {
    /// Add Third Party App
    /// 添加第三方应用
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamThirdPartyAppAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamThirdPartyAppServ::add_item(&mut add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Third Party App
    /// 修改第三方应用
    #[oai(path = "/:id", method = "put")]
    async fn modify(
        &self,
        id: Path<String>,
        mut modify_req: Json<IamThirdPartyAppModifyReq>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamThirdPartyAppServ::modify_item(&id.0, &mut modify_req, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Get Third Party App By Id
    /// 根据ID获取第三方应用
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamThirdPartyAppDetailResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamThirdPartyAppServ::get_item(&id.0, &IamThirdPartyAppFilterReq::default(), &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Delete Third Party App By Id
    /// 删除第三方应用
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamThirdPartyAppServ::delete_item_with_all_rels(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Get Third Party App By External Id
    /// 根据外部ID获取第三方应用
    #[oai(path = "/by_external_id/:external_id", method = "get")]
    async fn get_by_external_id(
        &self,
        external_id: Path<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Option<IamThirdPartyAppDetailResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamThirdPartyAppServ::get_item_by_external_id(&external_id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Delete Third Party App By External Id
    /// 根据外部ID删除第三方应用
    #[oai(path = "/by_external_id/:external_id", method = "delete")]
    async fn delete_by_external_id(
        &self,
        external_id: Path<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamThirdPartyAppServ::delete_item_by_external_id(&external_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Paginate Third Party Apps
    /// 分页查询第三方应用
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        id: Query<Option<String>>,
        ids: Query<Option<String>>,
        name: Query<Option<String>>,
        external_id: Query<Option<String>>,
        status: Query<Option<IamThirdPartyAppStatusKind>>,
        scope_level: Query<Option<bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind>>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamThirdPartyAppSummaryResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let ids = id.0
            .map(|id| vec![id])
            .or_else(|| ids.0.map(|s| s.split(',').map(str::to_string).collect::<Vec<String>>()));

        let result = IamThirdPartyAppServ::paginate_items(
            &IamThirdPartyAppFilterReq {
                basic: RbumBasicFilterReq {
                    ids,
                    name: name.0,
                    scope_level: scope_level.0,
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                external_id: external_id.0,
                status: status.0,
                ..Default::default()
            },
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Bind Account To Third Party App
    /// 绑定账号到第三方应用
    #[oai(path = "/:id/account/:account_id", method = "put")]
    async fn add_rel_account(
        &self,
        id: Path<String>,
        account_id: Path<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamThirdPartyAppServ::add_rel_account(&id.0, &account_id.0, false, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Unbind Account From Third Party App
    /// 解绑账号与第三方应用
    #[oai(path = "/:id/account/:account_id", method = "delete")]
    async fn delete_rel_account(
        &self,
        id: Path<String>,
        account_id: Path<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamThirdPartyAppServ::delete_rel_account(&id.0, &account_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch Bind Accounts To Third Party App
    /// 批量绑定账号到第三方应用
    #[oai(path = "/:id/account/batch/:account_ids", method = "put")]
    async fn batch_add_rel_account(
        &self,
        id: Path<String>,
        account_ids: Path<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        join_all(
            account_ids
                .0
                .split(',')
                .map(|account_id| async { IamThirdPartyAppServ::add_rel_account(&id.0, account_id, false, &funs, &ctx.0).await })
                .collect_vec(),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<()>, TardisError>>()?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Batch Unbind Accounts From Third Party App
    /// 批量解绑账号与第三方应用
    #[oai(path = "/:id/account/batch/:account_ids", method = "delete")]
    async fn batch_delete_rel_account(
        &self,
        id: Path<String>,
        account_ids: Path<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        for account_id in account_ids.0.split(',') {
            IamThirdPartyAppServ::delete_rel_account(&id.0, account_id, &funs, &ctx.0).await?;
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Find Accounts Bound To Third Party App
    /// 查询第三方应用绑定的账号列表
    #[oai(path = "/:id/account", method = "get")]
    async fn find_rel_account(
        &self,
        id: Path<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<RbumRelAggResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamThirdPartyAppServ::find_rel_account(&id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Third Party Apps Bound To Account
    /// 获取账号所关联的所有第三方应用
    #[oai(path = "/by_account/:account_id", method = "get")]
    async fn find_rel_third_party_app(
        &self,
        account_id: Path<String>,
        /// 是否可见：true-筛选ext.visible为true或ext为null；false-筛选ext.visible为false；不传则忽略
        visible: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<IamThirdPartyAppSummaryResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamThirdPartyAppServ::find_rel_third_party_app(&account_id.0, visible.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Paginate Accounts Bound To Third Party App
    /// 分页查询第三方应用绑定的账号
    #[oai(path = "/:id/account/paginate", method = "get")]
    async fn paginate_rel_account(
        &self,
        id: Path<String>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<RbumRelBoneResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamThirdPartyAppServ::paginate_rel_account(
            &id.0,
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}
