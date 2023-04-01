use crate::basic::dto::iam_filer_dto::{IamAccountFilterReq, IamAppFilterReq, IamTenantFilterReq};
use bios_basic::process::task_processor::TaskProcessor;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::web_resp::{TardisApiResult, TardisResp};
use tardis::TardisFunsInst;

use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
#[cfg(feature = "spi_kv")]
use crate::basic::serv::spi_client::spi_kv_client::SpiKvClient;
use crate::iam_config::IamConfig;
use crate::iam_constants;

pub struct IamCsSpiDataApi;

/// System Console Tenant API
#[poem_openapi::OpenApi(prefix_path = "/cs/init/data", tag = "bios_basic::ApiTag::System")]
impl IamCsSpiDataApi {
    /// Do Init Data
    #[oai(path = "/", method = "put")]
    async fn init_spi_data(&self, ctx: TardisContextExtractor) -> TardisApiResult<Option<String>> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        Self::do_init_spi_data(&funs, &ctx.0).await?;
        funs.commit().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0)? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }
    async fn do_init_spi_data(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        #[cfg(feature = "spi_kv")]
        {
            let task_ctx = ctx.clone();
            TaskProcessor::execute_task_with_ctx(
                &funs.conf::<IamConfig>().cache_key_async_task_status,
                move || async move {
                    let mut funs = iam_constants::get_tardis_inst();
                    funs.begin().await?;
                    //app kv
                    let mut next = true;
                    let mut i = 1;
                    while next {
                        let page = IamAppServ::paginate_items(
                            &IamAppFilterReq {
                                basic: RbumBasicFilterReq {
                                    ignore_scope: false,
                                    rel_ctx_owner: false,
                                    own_paths: Some(task_ctx.own_paths.clone()),
                                    with_sub_own_paths: true,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            i,
                            100,
                            None,
                            None,
                            &funs,
                            &task_ctx,
                        )
                        .await?;
                        if page.total_size / 100 < i as u64 {
                            next = false;
                        }
                        i += 1;
                        for app in page.records {
                            SpiKvClient::add_or_modify_item(&app.id, &app.name.clone(), &funs, &task_ctx).await?;
                        }
                    }
                    //tenant kv
                    let mut next = true;
                    let mut i = 1;
                    while next {
                        let page = IamTenantServ::paginate_items(
                            &IamTenantFilterReq {
                                basic: RbumBasicFilterReq {
                                    ignore_scope: false,
                                    rel_ctx_owner: false,
                                    own_paths: Some(task_ctx.own_paths.clone()),
                                    with_sub_own_paths: true,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            i,
                            100,
                            None,
                            None,
                            &funs,
                            &task_ctx,
                        )
                        .await?;
                        if page.total_size / 100 < i as u64 {
                            next = false;
                        }
                        i += 1;
                        for tenant in page.records {
                            SpiKvClient::add_or_modify_item(&tenant.id, &tenant.name.clone(), &funs, &task_ctx).await?;
                        }
                    }
                    //account kv
                    let mut next = true;
                    let mut i = 1;
                    while next {
                        let page = IamAccountServ::paginate_items(
                            &IamAccountFilterReq {
                                basic: RbumBasicFilterReq {
                                    ignore_scope: false,
                                    rel_ctx_owner: false,
                                    own_paths: Some(task_ctx.own_paths.clone()),
                                    with_sub_own_paths: true,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            i,
                            100,
                            None,
                            None,
                            &funs,
                            &task_ctx,
                        )
                        .await?;
                        if page.total_size / 100 < i as u64 {
                            next = false;
                        }
                        i += 1;
                        for account in page.records {
                            IamAccountServ::add_or_modify_account_search(&account.id, false, &funs, &task_ctx).await?;
                        }
                    }
                    funs.commit().await?;
                    Ok(())
                },
                funs,
                ctx,
            )
            .await?;
        }
        Ok(())
    }
}
