use std::collections::HashMap;

use bios_sdk_invoke::clients::spi_log_client::LogItemFindResp;
use serde::{Deserialize, Serialize};
use tardis::web::poem_openapi;

use super::{flow_inst_dto::FlowInstDetailResp, flow_model_dto::FlowModelDetailResp, flow_state_dto::FlowStateDetailResp};

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowSubDeployOneExportAggResp {
    pub states: Vec<FlowStateDetailResp>,
    pub models: Vec<FlowModelDetailResp>,
    pub delete_logs: HashMap<String, Option<Vec<LogItemFindResp>>>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowSubDeployTowImportReq {
    pub states: Vec<FlowStateDetailResp>,
    pub models: Vec<FlowModelDetailResp>,
    pub delete_logs: HashMap<String, Option<Vec<LogItemFindResp>>>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowSubDeployTowExportAggResp {
    pub insts: Vec<FlowInstDetailResp>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowSubDeployOneImportReq {
    pub insts: Option<Vec<FlowInstDetailResp>>,
}