use bios_basic::process::task_processor::TaskProcessor;

use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::{poem, poem_openapi};
use tardis::web::poem_openapi::payload::Attachment;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_filer_dto::{IamSubDeployFilterReq, IamSubDeployHostFilterReq, IamSubDeployLicenseFilterReq};
use crate::basic::dto::iam_sub_deploy_dto::{IamSubDeployAddReq, IamSubDeployDetailResp, IamSubDeployModifyReq, IamSubDeploySummaryResp};
use crate::basic::dto::iam_sub_deploy_host_dto::{IamSubDeployHostAddReq, IamSubDeployHostDetailResp, IamSubDeployHostModifyReq};
use crate::basic::dto::iam_sub_deploy_license_dto::{IamSubDeployLicenseAddReq, IamSubDeployLicenseDetailResp, IamSubDeployLicenseModifyReq};
use crate::basic::serv::iam_sub_deploy_serv::{IamSubDeployHostServ, IamSubDeployLicenseServ, IamSubDeployServ};
use crate::iam_constants::{self, RBUM_SCOPE_LEVEL_GLOBAL};
use crate::iam_enumeration::IamSubDeployHostKind;

use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCsSubDeployApi;
#[derive(Clone, Default)]
pub struct IamCsSubDeployHostApi;
#[derive(Clone, Default)]
pub struct IamCsSubDeployLicenseApi;

/// System Console Sub Deploy API
/// 系统控制台 二级部署 API
#[poem_openapi::OpenApi(prefix_path = "/cs/sub_deploy", tag = "bios_basic::ApiTag::System")]
impl IamCsSubDeployApi {
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamSubDeployAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        add_req.0.scope_level = Some(RBUM_SCOPE_LEVEL_GLOBAL);
        let result = IamSubDeployServ::add_item(&mut add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamSubDeployModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployServ::modify_item(&id.0, &mut modify_req.0, &funs, &ctx.0).await?;
        // todo 批量修改extends access_url
        if let Some(access_url) = modify_req.0.access_url {
            let sub_deploy_extends_ids = IamSubDeployServ::find_id_items(
                &IamSubDeployFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    extend_sub_deploy_id: Some(id.0),
                    ..Default::default()
                },
                None,
                None,
                &funs,
                &ctx.0,
            )
            .await?;
            for sub_deploy_extends_id in sub_deploy_extends_ids {
                IamSubDeployServ::modify_item(
                    &sub_deploy_extends_id,
                    &mut IamSubDeployModifyReq {
                        access_url: Some(access_url.clone()),
                        name: None,
                        province: None,
                        note: None,
                        scope_level: None,
                        disabled: None,
                    },
                    &funs,
                    &ctx.0,
                )
                .await?;
            }
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Get Sub Deploy By Sub Deploy Id
    /// 根据二级部署ID获取二级部署
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamSubDeployDetailResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployServ::get_item(&id.0, &IamSubDeployFilterReq::default(), &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Sub Deploy
    /// 查找二级部署
    #[allow(clippy::too_many_arguments)]
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        ids: Query<Option<String>>,
        name: Query<Option<String>>,
        code: Query<Option<String>>,
        province: Query<Option<String>>,
        access_url: Query<Option<String>>,
        enabled: Query<Option<bool>>,
        extend_sub_deploy_id: Query<Option<String>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamSubDeploySummaryResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployServ::paginate_items(
            &IamSubDeployFilterReq {
                basic: RbumBasicFilterReq {
                    ids: ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                    name: name.0,
                    code: code.0,
                    with_sub_own_paths: with_sub.0.unwrap_or(false),
                    enabled: enabled.0,
                    ..Default::default()
                },
                province: province.0,
                access_url: access_url.0,
                extend_sub_deploy_id: Some(extend_sub_deploy_id.0.unwrap_or("".to_string())),
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

    /// Delete Sub Deploy By Sub Deploy Id
    /// 删除二级部署
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployServ::delete_item_with_ext_rel(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }
}

/// Tenant Console Sub Deploy Host API
///
/// 租户控制台 二级部署 地址 API
#[poem_openapi::OpenApi(prefix_path = "/cs/sub_deploy/host", tag = "bios_basic::ApiTag::System")]
impl IamCsSubDeployHostApi {
    /// Get Sub Deploy Host By Sub Deploy Host Id
    ///
    /// 根据二级部署地址ID获取二级部署地址
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamSubDeployHostDetailResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployHostServ::get_rbum(
            &id.0,
            &IamSubDeployHostFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
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

    /// Find Sub Deploy Host
    ///
    /// 查找二级部署地址
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        ids: Query<Option<String>>,
        sub_deploy_id: Query<Option<String>>,
        host: Query<Option<String>>,
        host_type: Query<Option<IamSubDeployHostKind>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamSubDeployHostDetailResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployHostServ::paginate_rbums(
            &IamSubDeployHostFilterReq {
                basic: RbumBasicFilterReq {
                    ids: ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                sub_deploy_id: sub_deploy_id.0,
                host: host.0,
                host_type: host_type.0,
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

    /// Add Sub Deploy Host
    ///
    /// 添加二级部署地址
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamSubDeployHostAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamSubDeployHostServ::add_rbum(&mut add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Sub Deploy Host By Sub Deploy Host Id
    ///
    /// 修改二级部署地址
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamSubDeployHostModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployHostServ::modify_rbum(&id.0, &mut modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Sub Deploy Host By Sub Deploy Host Id
    ///
    /// 删除二级部署地址
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployHostServ::delete_rbum(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }
}

/// Tenant Console Sub Deploy License API
///
/// 租户控制台 二级部署 许可证 API
#[poem_openapi::OpenApi(prefix_path = "/cs/sub_deploy/license", tag = "bios_basic::ApiTag::System")]
impl IamCsSubDeployLicenseApi {
    /// Get Sub Deploy License By Sub Deploy Id
    ///
    /// 根据二级部署ID获取二级部署许可证
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamSubDeployLicenseDetailResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployLicenseServ::get_rbum(
            &id.0,
            &IamSubDeployLicenseFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
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

    /// Find Sub Deploy License
    ///
    /// 查找二级部署许可证
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        ids: Query<Option<String>>,
        sub_deploy_id: Query<String>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamSubDeployLicenseDetailResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployLicenseServ::paginate_rbums(
            &IamSubDeployLicenseFilterReq {
                basic: RbumBasicFilterReq {
                    ids: ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                sub_deploy_id: Some(sub_deploy_id.0),
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

    /// Show License
    ///
    /// 显示许可证
    #[oai(path = "/show/license/:id", method = "get")]
    async fn show_license(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployLicenseServ::show_license(
            &id.0,
            &IamSubDeployLicenseFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
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

    /// Download License
    ///
    /// 下载许可证
    #[oai(path = "/download/license/:id", method = "get")]
    async fn download_license(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> Result<Attachment<Vec<u8>>, poem::Error> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamSubDeployLicenseServ::generate_license(&id.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        let attachment = Attachment::new(result.into_bytes()).filename("license.cert");
        Ok(attachment.into())
    }

    /// Add Sub Deploy License
    ///
    /// 添加二级部署许可证
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamSubDeployLicenseAddReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamSubDeployLicenseServ::add_rbum(&mut add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Modify Sub Deploy License By Sub Deploy Id
    ///
    /// 修改二级部署许可证
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamSubDeployLicenseModifyReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployLicenseServ::modify_rbum(&id.0, &mut modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Delete Sub Deploy License By Sub Deploy Id
    ///
    /// 删除二级部署许可证
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Option<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamSubDeployLicenseServ::delete_rbum(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }
}