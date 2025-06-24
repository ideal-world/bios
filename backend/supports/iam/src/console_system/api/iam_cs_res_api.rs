use std::collections::HashMap;

use crate::basic::dto::iam_filer_dto::IamResFilterReq;
use crate::basic::dto::iam_res_dto::{IamResAggAddAndBindReq, IamResAggAddReq, IamResDetailResp, IamResModifyReq, IamResSummaryResp};
use crate::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq, IamSetTreeResp};
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_res_serv::IamResServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use crate::iam_enumeration::{IamRelKind, IamResKind, IamSetKind};
use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumSetCateFilterReq, RbumSetTreeFilterReq};
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeResp;
use bios_basic::rbum::rbum_config::RbumConfigApi;
use bios_basic::rbum::rbum_enumeration::RbumSetCateLevelQueryKind;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::rbum::serv::rbum_set_serv::RbumSetCateServ;
use itertools::Itertools;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};
use tardis::TardisFuns;

#[derive(Clone, Default)]
pub struct IamCsResApi;

/// System Console Res API
/// 系统控制台资源API
///
/// Note: the current res only supports sys level.
/// 注意：当前资源仅支持系统级别。
#[poem_openapi::OpenApi(prefix_path = "/cs/res", tag = "bios_basic::ApiTag::System")]
impl IamCsResApi {
    /// Add Res
    /// 添加资源
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamResAggAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Res, &funs, &ctx.0).await?;
        let result = IamResServ::add_res_agg(&mut add_req.0, &set_id, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Add And Bind Res
    /// 添加资源
    #[oai(path = "/add_and_bind", method = "post")]
    async fn add_and_bind(&self, add_req: Json<IamResAggAddAndBindReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let set_id = RbumSetCateServ::find_one_rbum(&RbumSetCateFilterReq {
            basic: RbumBasicFilterReq {
                own_paths: Some("".to_string()),
                with_sub_own_paths: true,
                ids: Some(vec![add_req.0.set.set_cate_id.clone()]),
                ..Default::default()
            },
            ..Default::default()
        }, &funs, &ctx.0)
        .await?
        .map(|s| s.rel_rbum_set_id)
        .ok_or_else(|| funs.err().not_found("iam_cs_res_api", "add_and_bind", &format!("not found set by set_cate_id {}", add_req.0.set.set_cate_id.clone()), "404-rbum-set-code-not-exist"))?;
        let result = IamResServ::add_res_agg(
            &mut IamResAggAddReq {
                res: add_req.0.res.clone(),
                set: add_req.0.set.clone(),
            },
            &set_id,
            &funs,
            &ctx.0,
        )
        .await?;
        IamRelServ::add_simple_rel(&add_req.0.bind_res_kind, &result, &add_req.0.bind_res_id, None, None, false, false, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Add Res Cate
    /// 添加资源分类
    #[oai(path = "/cate", method = "post")]
    async fn add_cate(&self, add_req: Json<IamSetCateAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let set_cate_sys_code_node_len = funs.rbum_conf_set_cate_sys_code_node_len();
        let api_sys_codes = TardisFuns::field.incr_by_base36(&String::from_utf8(vec![b'0'; set_cate_sys_code_node_len]).unwrap_or_default()).map(|api_sys_code| vec![api_sys_code]);
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Res, &funs, &ctx.0).await?;
        let rbum_parent_cate_id = if add_req.0.rbum_parent_cate_id.is_none() {
            Some(IamSetServ::get_cate_id_with_sys_code(set_id.as_str(), api_sys_codes, &funs, &ctx.0).await?)
        } else {
            add_req.0.rbum_parent_cate_id
        };
        let result = IamSetServ::add_set_cate(
            &set_id,
            &IamSetCateAddReq {
                name: add_req.0.name,
                scope_level: add_req.0.scope_level,
                bus_code: add_req.0.bus_code,
                icon: add_req.0.icon,
                sort: add_req.0.sort,
                ext: add_req.0.ext,
                rbum_parent_cate_id,
            },
            &funs,
            &ctx.0,
        )
        .await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Add Api Res To Res
    /// 添加资源
    #[oai(path = "/:id/res/:res_api_id", method = "put")]
    async fn add_rel_res(&self, id: Path<String>, res_api_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRelServ::add_simple_rel(&IamRelKind::IamResApi, &res_api_id.0, &id.0, None, None, false, false, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Count Api Res By Res Id
    /// 统计资源
    #[oai(path = "/:id/res/total", method = "get")]
    async fn count_rel_res(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<u64> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamRelServ::count_to_rels(&IamRelKind::IamResApi, &id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Delete Res By Res Id
    /// 删除资源
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamResServ::delete_res(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Res Cate By Res Cate Id
    /// 删除资源分类
    #[oai(path = "/cate/:id", method = "delete")]
    async fn delete_cate(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::delete_set_cate(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Api Res By Res Id
    /// 删除资源
    #[oai(path = "/:id/res/:res_api_id", method = "delete")]
    async fn delete_rel_res(&self, id: Path<String>, res_api_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamRelServ::delete_simple_rel(&IamRelKind::IamResApi, &res_api_id.0, &id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Find Res Api
    /// 查找资源
    #[oai(path = "/", method = "get")]
    async fn find(
        &self,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<IamResSummaryResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamResServ::find_items(
            &IamResFilterReq {
                kind: Some(IamResKind::Api),
                ..Default::default()
            },
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// page Res Api
    /// 分页查找资源
    #[oai(path = "/page", method = "get")]
    async fn paginate(
        &self,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamResSummaryResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamResServ::paginate_items(
            &IamResFilterReq {
                kind: Some(IamResKind::Api),
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

    /// Find Api Res By Res Id
    /// 查找资源
    #[oai(path = "/:id/res", method = "get")]
    async fn find_rel_res(
        &self,
        id: Path<String>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<IamResDetailResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamResServ::find_rel_res(&IamRelKind::IamResApi, &id.0, desc_by_create.0, desc_by_update.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Rel Roles By Res Id
    /// 查找关联角色
    #[oai(path = "/:id/role", method = "get")]
    async fn find_rel_roles(
        &self,
        id: Path<String>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<RbumRelBoneResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamResServ::find_from_simple_rel_roles(&IamRelKind::IamResRole, false, &id.0, desc_by_create.0, desc_by_update.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Element Rel Apis By Res Ids
    /// 查找关联资源
    #[oai(path = "/get_res_apis/:ids", method = "get")]
    async fn get_res_apis(&self, ids: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<HashMap<String, Vec<IamResDetailResp>>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let id_list = ids.split(',').collect_vec();
        let result = IamResServ::find_to_multi_rel_roles(&IamRelKind::IamResApi, id_list, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Get Res By Res Id
    /// 获取资源
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamResDetailResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamResServ::get_item(&id.0, &IamResFilterReq::default(), &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Api Tree
    /// 查找资源树
    #[oai(path = "/tree/api", method = "get")]
    async fn get_api_tree(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<RbumSetTreeResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Res, &funs, &ctx.0).await?;
        let result = IamSetServ::get_api_tree(&set_id, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Menu Tree
    /// 查找菜单树
    #[oai(path = "/tree/menu", method = "get")]
    async fn get_menu_tree(&self, exts: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamSetTreeResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Res, &funs, &ctx.0).await?;
        let result = IamSetServ::get_menu_tree(&set_id, exts.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Data Guard Tree
    /// 查找数据权限树
    #[oai(path = "/tree/data_guard", method = "get")]
    async fn get_data_guard_tree(&self, exts: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<RbumSetTreeResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::DataGuard, &funs, &ctx.0).await?;
        let result = IamSetServ::get_data_guard_tree(&set_id, exts.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Res Tree
    /// 查找资源树
    ///
    /// * Without parameters: Query the whole tree
    /// * ``parent_sys_code=true`` : query only the next level. This can be used to query level by level when the tree is too large
    ///
    /// * 无参数：查询整个树
    /// * ``parent_sys_code=true``：仅查询下一级。当树太大时，可以用于逐级查询
    #[oai(path = "/tree", method = "get")]
    async fn get_tree(&self, parent_sys_code: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<RbumSetTreeResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Res, &funs, &ctx.0).await?;
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
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Res By Res Id
    /// 修改资源
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamResModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamResServ::modify_item(&id.0, &mut modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Modify Res Cate By Res Cate Id
    /// 修改资源分类
    #[oai(path = "/cate/:id", method = "put")]
    async fn modify_set_cate(&self, id: Path<String>, modify_req: Json<IamSetCateModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::modify_set_cate(&id.0, &modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Modify Parent Cate Id By Res Cate Id
    ///
    #[oai(path = "/cate/:id/parent/:parent_cate_id", method = "put")]
    async fn move_set_cate(&self, id: Path<String>, parent_cate_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSetServ::move_set_cate(&id.0, &parent_cate_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }
}
