use bios_sdk_invoke::{clients::spi_kv_client::SpiKvClient, invoke_constants::TARDIS_CONTEXT};
use itertools::Itertools;
use serde_json::Value;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    web::web_resp::TardisResp,
    TardisFuns, TardisFunsInst,
};

use crate::{
    dto::{
        flow_external_dto::{
            FlowExternalFetchRelObjReq, FlowExternalFetchRelObjResp, FlowExternalKind, FlowExternalModifyFieldReq, FlowExternalModifyFieldResp, FlowExternalNotifyChangesReq,
            FlowExternalNotifyChangesResp, FlowExternalParams, FlowExternalReq,
        },
        flow_transition_dto::FlowTransitionActionByVarChangeInfo,
    },
    flow_constants,
};

pub struct FlowExternalServ;

impl FlowExternalServ {
    pub async fn do_fetch_rel_obj(
        tag: &str,
        rel_business_obj_id: &str,
        rel_tags: Vec<String>,
        ctx: &TardisContext,
        funs: &TardisFunsInst,
    ) -> TardisResult<FlowExternalFetchRelObjResp> {
        let external_url = Self::get_external_url(tag, ctx, funs).await?;
        let header = Self::headers(None, funs, ctx).await?;
        let resp: TardisResp<FlowExternalFetchRelObjResp> = funs
            .web_client()
            .post(
                &external_url,
                &FlowExternalReq {
                    kind: FlowExternalKind::FetchRelObj,
                    curr_tag: tag.to_string(),
                    curr_bus_obj_id: rel_business_obj_id.to_string(),
                    target_state: None,
                    params: rel_tags.into_iter().map(|tag| FlowExternalParams::FetchRelObj(FlowExternalFetchRelObjReq { rel_tag: tag })).collect_vec(),
                },
                header,
            )
            .await?
            .body
            .ok_or_else(|| funs.err().internal_error("flow_external", "do_fetch_rel_obj", "illegal response", "500-external-illegal-response"))?;
        if let Some(data) = resp.data {
            Ok(data)
        } else {
            Err(funs.err().internal_error("flow_external", "do_fetch_rel_obj", "illegal response", "500-external-illegal-response"))
        }
    }

    pub async fn do_modify_field(
        tag: &str,
        rel_business_obj_id: &str,
        change_info: &FlowTransitionActionByVarChangeInfo,
        ctx: &TardisContext,
        funs: &TardisFunsInst,
    ) -> TardisResult<FlowExternalModifyFieldResp> {
        let external_url = Self::get_external_url(tag, ctx, funs).await?;
        if external_url.is_empty() {
            return Ok(FlowExternalModifyFieldResp {});
        }

        let header = Self::headers(None, funs, ctx).await?;
        let resp: TardisResp<FlowExternalModifyFieldResp> = funs
            .web_client()
            .post(
                &external_url,
                &FlowExternalReq {
                    kind: FlowExternalKind::ModifyField,
                    curr_tag: tag.to_string(),
                    curr_bus_obj_id: rel_business_obj_id.to_string(),
                    target_state: None,
                    params: vec![FlowExternalParams::ModifyField(FlowExternalModifyFieldReq {
                        var_id: Some(change_info.var_name.clone()),
                        var_name: Some(change_info.var_name.clone()),
                        value: change_info.changed_val.clone(),
                    })],
                },
                header,
            )
            .await?
            .body
            .ok_or_else(|| funs.err().internal_error("flow_external", "do_modify_field", "illegal response", "500-external-illegal-response"))?;
        if let Some(data) = resp.data {
            Ok(data)
        } else {
            Err(funs.err().internal_error("flow_external", "do_modify_field", "illegal response", "500-external-illegal-response"))
        }
    }

    pub async fn do_notify_changes(
        tag: &str,
        rel_business_obj_id: &str,
        target_state: Option<String>,
        changes: Vec<Value>,
        ctx: &TardisContext,
        funs: &TardisFunsInst,
    ) -> TardisResult<FlowExternalNotifyChangesResp> {
        let external_url = Self::get_external_url(tag, ctx, funs).await?;
        if external_url.is_empty() {
            return Ok(FlowExternalNotifyChangesResp {});
        }

        let header = Self::headers(None, funs, ctx).await?;
        let resp: TardisResp<FlowExternalNotifyChangesResp> = funs
            .web_client()
            .post(
                &external_url,
                &FlowExternalReq {
                    kind: FlowExternalKind::NotifyChanges,
                    curr_tag: tag.to_string(),
                    curr_bus_obj_id: rel_business_obj_id.to_string(),
                    target_state,
                    params: vec![FlowExternalParams::NotifyChanges(FlowExternalNotifyChangesReq { changed_vars: changes })],
                },
                header,
            )
            .await?
            .body
            .ok_or_else(|| funs.err().internal_error("flow_external", "do_notify_changes", "illegal response", "500-external-illegal-response"))?;
        if let Some(data) = resp.data {
            Ok(data)
        } else {
            Err(funs.err().internal_error("flow_external", "do_notify_changes", "illegal response", "500-external-illegal-response"))
        }
    }

    async fn get_external_url(tag: &str, ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<String> {
        let external_url = SpiKvClient::get_item(format!("{}:config:{}", flow_constants::DOMAIN_CODE, tag), None, funs, ctx)
            .await?
            .ok_or_else(|| funs.err().not_found("flow_external", "get_external_url", "not found external url", "404-external-data-url-not-exist"))?;
        Ok(external_url.value.as_str().unwrap_or_default().to_string())
    }

    async fn headers(headers: Option<Vec<(String, String)>>, _funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<Vec<(String, String)>>> {
        let base_ctx = (TARDIS_CONTEXT.to_string(), TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(ctx)?));
        if let Some(mut headers) = headers {
            headers.push(base_ctx);
            return Ok(Some(headers));
        }
        let headers = Some(vec![base_ctx]);
        Ok(headers)
    }
}
