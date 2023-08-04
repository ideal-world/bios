use bios_mw_flow::dto::flow_external_dto::{FlowExternalFetchRelObjResp, FlowExternalKind, FlowExternalModifyFieldResp, FlowExternalNotifyChangesResp, FlowExternalReq};
use serde_json::{json, Value};
use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::web::Json,
    poem_openapi,
    web_resp::{TardisApiResult, TardisResp},
};

#[derive(Clone)]
pub struct MockApi;

/// Flow Config process API
#[poem_openapi::OpenApi(prefix_path = "/mock")]
impl MockApi {
    /// Exchange Data / 数据交换
    #[oai(path = "/exchange_data", method = "post")]
    async fn external_data(&self, req: Json<FlowExternalReq>, _ctx: TardisContextExtractor) -> TardisApiResult<Value> {
        let result = match req.0.kind {
            FlowExternalKind::FetchRelObj => {
                json!(FlowExternalFetchRelObjResp {
                    rel_bus_obj_ids: vec!["mock-rel-obj-id".to_string()],
                })
            }
            FlowExternalKind::ModifyField => {
                json!(FlowExternalModifyFieldResp { rel_bus_obj_ids: vec![] })
            }
            FlowExternalKind::NotifyChanges => {
                json!(FlowExternalNotifyChangesResp {})
            }
        };
        TardisResp::ok(result)
    }
}
