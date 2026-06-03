use std::collections::HashSet;

use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp};

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_app_dto::IamAppSummaryResp;
use crate::basic::dto::iam_filer_dto::IamAppFilterReq;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::iam_constants;

#[derive(Clone, Default)]
pub struct IamCcAppApi;

/// Common Console App API
/// 通用控制台应用API
#[poem_openapi::OpenApi(prefix_path = "/cc/app", tag = "bios_basic::ApiTag::Common")]
impl IamCcAppApi {
    /// Find Apps
    /// 查找应用
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        id: Query<Option<String>>,
        ids: Query<Option<String>>,
        name: Query<Option<String>>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamAppSummaryResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();

        let ids = id.0.map(|id| vec![id]).or(ids.0.map(|ids| ids.split(',').map(|s| s.to_string()).collect()));
        let result = IamAppServ::paginate_items(
            &IamAppFilterReq {
                basic: RbumBasicFilterReq {
                    ids,
                    name: name.0,
                    with_sub_own_paths: true,
                    enabled: Some(true),
                    ..Default::default()
                },
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

    /// Find App Name and icon By Ids
    /// 根据应用ID查找应用名称和图标
    ///
    /// Return format: ["<id>,<name>,<icon>"]
    /// 返回格式：["<id>,<name>,<icon>"]
    #[oai(path = "/name", method = "get")]
    async fn find_name_by_ids(
        &self,
        // App Ids, multiple ids separated by ,
        ids: Query<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let ids = ids.0.split(',').map(|s| s.to_string()).collect();
        let result = IamAppServ::find_name_by_ids(
            IamAppFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(ids),
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ignore_scope: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// [临时脚本] 批量删除应用（按保留列表同步）
    ///
    /// 入参 `app_ids` 为系统中应保留的应用 ID 列表。
    /// 查询当前上下文范围内全部应用，对不在该列表中的应用执行删除（IAM 应用仅支持禁用，等同删除）。
    ///
    /// 返回已成功禁用的应用 ID 列表。
    #[oai(path = "/script/batch-delete", method = "post")]
    async fn script_batch_delete_apps(&self, app_ids: Json<Vec<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;

        let keep_ids: HashSet<String> = app_ids.0.into_iter().collect();
        let all_apps = IamAppServ::find_items(
            &IamAppFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            None,
            &funs,
            &ctx.0,
        )
        .await?;

        let deleted_ids: Vec<String> = all_apps.into_iter().filter(|app| !keep_ids.contains(&app.id) && !app.disabled).map(|app| app.id).collect();

        if !deleted_ids.is_empty() {
            let in_clause = deleted_ids.iter().map(|id| format!("'{}'", id.replace('\'', "''"))).collect::<Vec<_>>().join(",");
            let sql = format!("UPDATE rbum_item SET disabled = true WHERE id IN ({in_clause}) AND disabled = false");
            funs.db().execute_one(&sql, vec![]).await?;
        }

        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(deleted_ids)
    }
}
