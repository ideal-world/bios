use std::collections::HashMap;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_sdk_invoke::clients::spi_kv_client::KvItemSummaryResp;
use bios_sdk_invoke::clients::spi_search_client::SpiSearchClient;
use bios_sdk_invoke::dto::search_item_dto::SearchItemModifyReq;
use serde_json::json;
use tardis::basic::dto::TardisContext;
use tardis::tokio;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::flow_config_dto::FlowConfigModifyReq;

use crate::dto::flow_state_dto::FlowStateFilterReq;
use crate::flow_constants;
use crate::serv::flow_config_serv::FlowConfigServ;
use crate::serv::flow_inst_serv::FlowInstServ;
use crate::serv::flow_state_serv::FlowStateServ;
#[derive(Clone)]
pub struct FlowCsConfigApi;

/// Flow Config process API
#[poem_openapi::OpenApi(prefix_path = "/cs/config")]
impl FlowCsConfigApi {
    /// Modify Config / 编辑配置
    #[oai(path = "/", method = "post")]
    async fn modify_config(&self, req: Json<Vec<FlowConfigModifyReq>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        FlowConfigServ::modify_config(&req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Get Config / 获取配置
    #[oai(path = "/", method = "get")]
    async fn get(&self, ctx: TardisContextExtractor) -> TardisApiResult<Option<TardisPage<KvItemSummaryResp>>> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let result = FlowConfigServ::get_config(&funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Get Config / 获取配置
    #[oai(path = "/update_instance_state", method = "get")]
    async fn update_instance_state(&self, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        tokio::spawn(async move {
            let global_ctx = TardisContext::default();
            let funs = flow_constants::get_tardis_inst();
            let tag_search_map = HashMap::from([
                ("CTS", "idp_test"),
                ("ISSUE", "idp_test"),
                ("ITER", "idp_project"),
                ("MS", "idp_project"),
                ("PROJ", "idp_project"),
                ("REQ", "idp_project"),
                ("TASK", "idp_project"),
                ("TICKET", "ticket"),
                ("TP", "idp_test"),
                ("TS", "idp_test"),
            ]);
            let states = FlowStateServ::find_id_name_items(
                &FlowStateFilterReq {
                    basic: RbumBasicFilterReq {
                        ignore_scope: true,
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                None,
                None,
                &funs,
                &global_ctx,
            )
            .await
            .unwrap();
            let mut page = 1;
            loop {
                let insts = FlowInstServ::paginate(None, None, None, None, Some(true), page, 200, &funs, &global_ctx).await.unwrap().records;
                if insts.is_empty() {
                    break;
                }
                for inst in insts {
                    let state_name = states.get(&inst.current_state_id).cloned().unwrap_or_default();
                    if let Some(table) = tag_search_map.get(&inst.tag.as_str()) {
                        SpiSearchClient::modify_item_and_name(table, &inst.rel_business_obj_id, &SearchItemModifyReq {
                            kind: None,
                            title: None,
                            name: None,
                            content: None,
                            owner: None,
                            own_paths: None,
                            create_time: None,
                            update_time: None,
                            ext: Some(json!({
                                "status": state_name,
                            })),
                            ext_override: None,
                            visit_keys: None,
                            kv_disable: None,
                        }, &funs, &global_ctx).await.unwrap_or_default();
                    }
                }
                page += 1;
            }
        });
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }
}
