use bios_mw_flow::dto::flow_external_dto::{
    FlowExternalFetchRelObjResp, FlowExternalKind, FlowExternalModifyFieldResp, FlowExternalNotifyChangesResp, FlowExternalReq, RelBusObjResp,
};
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
    async fn exchange_data(&self, req: Json<FlowExternalReq>, _ctx: TardisContextExtractor) -> TardisApiResult<Value> {
        let result = match req.0.kind {
            FlowExternalKind::FetchRelObj => match req.curr_tag.as_str() {
                "REQ" => {
                    json!(FlowExternalFetchRelObjResp {
                        curr_tag: req.curr_tag.clone(),
                        curr_bus_obj_id: req.curr_bus_obj_id.clone(),
                        rel_bus_objs: vec![RelBusObjResp {
                            rel_tag: "TICKET".to_string(),
                            rel_bus_obj_ids: vec!["mock-ticket-obj-id".to_string()],
                        },],
                    })
                }
                "TICKET" => json!(FlowExternalFetchRelObjResp {
                    curr_tag: req.curr_tag.clone(),
                    curr_bus_obj_id: req.curr_bus_obj_id.clone(),
                    rel_bus_objs: vec![RelBusObjResp {
                        rel_tag: "ITER".to_string(),
                        rel_bus_obj_ids: vec!["mock-iter-obj-id".to_string()],
                    },],
                }),
                _ => json!({}),
            },
            FlowExternalKind::ModifyField => {
                json!(FlowExternalModifyFieldResp {})
            }
            FlowExternalKind::NotifyChanges => {
                json!(FlowExternalNotifyChangesResp {})
            }
        };
        TardisResp::ok(result)
    }
}
