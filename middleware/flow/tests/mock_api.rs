use bios_mw_flow::dto::flow_external_dto::{
    FlowExternalFetchRelObjResp, FlowExternalKind, FlowExternalModifyFieldResp, FlowExternalNotifyChangesResp, FlowExternalReq, RelBusObjResp,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tardis::{
    basic::{
        error::TardisError,
        result::{TARDIS_RESULT_ACCEPTED_CODE, TARDIS_RESULT_SUCCESS_CODE},
    },
    web::{
        context_extractor::TardisContextExtractor,
        poem,
        poem_openapi::{
            self,
            payload::Json,
            types::{ParseFromJSON, ToJSON},
        },
    },
};

#[derive(Clone)]
pub struct MockApi;

/// Flow Config process API
#[poem_openapi::OpenApi(prefix_path = "/mock")]
impl MockApi {
    /// Exchange Data / 数据交换
    #[oai(path = "/exchange_data", method = "post")]
    async fn exchange_data(&self, req: Json<FlowExternalReq>, _ctx: TardisContextExtractor) -> MockApiResponse<Value> {
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
                        rel_bus_obj_ids: vec!["mock-iter-obj-id1".to_string(), "mock-iter-obj-id2".to_string()],
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
        MockResp::ok(result)
    }
}

pub type MockApiResponse<T> = poem::Result<Json<MockResp<T>>>;

#[derive(poem_openapi::Object, Deserialize, Serialize, Clone, Debug)]
pub struct MockResp<T>
where
    T: ParseFromJSON + ToJSON + Serialize + Send + Sync,
{
    pub code: String,
    pub message: String,
    pub body: Option<T>,
}

impl<T> MockResp<T>
where
    T: ParseFromJSON + ToJSON + Serialize + Send + Sync,
{
    pub fn ok(data: T) -> MockApiResponse<T> {
        MockApiResponse::Ok(Json(MockResp {
            code: TARDIS_RESULT_SUCCESS_CODE.to_string(),
            message: "".to_string(),
            body: Some(data),
        }))
    }

    pub fn accepted(data: T) -> MockApiResponse<T> {
        MockApiResponse::Ok(Json(MockResp {
            code: TARDIS_RESULT_ACCEPTED_CODE.to_string(),
            message: "".to_string(),
            body: Some(data),
        }))
    }

    pub fn err(error: TardisError) -> MockApiResponse<T> {
        MockApiResponse::Err(error.into())
    }
}
