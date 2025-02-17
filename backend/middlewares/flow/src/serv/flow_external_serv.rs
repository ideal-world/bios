use bios_sdk_invoke::{clients::spi_kv_client::SpiKvClient, invoke_constants::TARDIS_CONTEXT};
use itertools::Itertools;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::Utc,
    log::debug,
    tokio, TardisFuns, TardisFunsInst,
};

use crate::{
    dto::{
        flow_external_dto::{
            FlowExternalCallbackOp, FlowExternalDeleteRelObjResp, FlowExternalFetchRelObjResp, FlowExternalKind, FlowExternalModifyFieldResp, FlowExternalNotifyChangesResp,
            FlowExternalParams, FlowExternalQueryFieldResp, FlowExternalReq, FlowExternalResp,
        },
        flow_state_dto::FlowSysStateKind,
        flow_transition_dto::{FlowTransitionActionByVarChangeInfoChangedKind, FlowTransitionDetailResp, TagRelKind},
    },
    flow_constants,
};

pub struct FlowExternalServ;

impl FlowExternalServ {
    pub async fn do_fetch_rel_obj(
        tag: &str,
        inst_id: &str,
        rel_business_obj_id: &str,
        rel_tags: Vec<(String, Option<TagRelKind>)>,
        ctx: &TardisContext,
        funs: &TardisFunsInst,
    ) -> TardisResult<FlowExternalFetchRelObjResp> {
        let external_url = Self::get_external_url(tag, ctx, funs).await?;
        let header = Self::headers(None, funs, ctx).await?;
        let body = FlowExternalReq {
            kind: FlowExternalKind::FetchRelObj,
            inst_id: inst_id.to_string(),
            curr_tag: tag.to_string(),
            curr_bus_obj_id: rel_business_obj_id.to_string(),
            params: rel_tags
                .into_iter()
                .map(|(tag, kind)| FlowExternalParams {
                    rel_tag: Some(tag),
                    rel_kind: kind.map(String::from),
                    var_id: None,
                    var_name: None,
                    value: None,
                    changed_kind: None,
                    guard_conf: None,
                })
                .collect_vec(),
            sys_time: Some(Utc::now().timestamp_millis()),
            ..Default::default()
        };
        debug!("do_fetch_rel_obj body: {:?}", body);
        let resp: FlowExternalResp<FlowExternalFetchRelObjResp> = funs
            .web_client()
            .post(&external_url, &body, header)
            .await?
            .body
            .ok_or_else(|| funs.err().internal_error("flow_external", "do_fetch_rel_obj", "illegal response", "500-external-illegal-response"))?;
        if resp.code != *"200" {
            return Err(funs.err().internal_error("flow_external", "do_fetch_rel_obj", "illegal response", "500-external-illegal-response"));
        }
        if let Some(data) = resp.body {
            Ok(data)
        } else {
            Err(funs.err().internal_error("flow_external", "do_fetch_rel_obj", "illegal response", "500-external-illegal-response"))
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn do_async_modify_field(
        tag: &str,
        transition_detail: Option<FlowTransitionDetailResp>,
        rel_business_obj_id: &str,
        inst_id: &str,
        callback_op: Option<FlowExternalCallbackOp>,
        manual_op: Option<bool>,
        operator: Option<String>,
        target_state: Option<String>,
        target_sys_state: Option<FlowSysStateKind>,
        original_state: Option<String>,
        original_sys_state: Option<FlowSysStateKind>,
        params: Vec<FlowExternalParams>,
        ctx: &TardisContext,
        _funs: &TardisFunsInst,
    ) -> TardisResult<()> {
        let tag = tag.to_string();
        let rel_business_obj_id = rel_business_obj_id.to_string();
        let inst_id = inst_id.to_string();

        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            let funs = flow_constants::get_tardis_inst();
            let result = Self::do_modify_field(
                &tag,
                transition_detail,
                &rel_business_obj_id,
                &inst_id,
                callback_op,
                manual_op,
                operator,
                target_state,
                target_sys_state,
                original_state,
                original_sys_state,
                params,
                &ctx_clone,
                &funs,
            )
            .await;
            if let Err(err) = result {
                tardis::log::error!("[BIOS.Flow] failed to ModifyField event: {}", err);
            }
        });
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn do_modify_field(
        tag: &str,
        transition_detail: Option<FlowTransitionDetailResp>,
        rel_business_obj_id: &str,
        inst_id: &str,
        callback_op: Option<FlowExternalCallbackOp>,
        manual_op: Option<bool>,
        operator: Option<String>,
        target_state: Option<String>,
        target_sys_state: Option<FlowSysStateKind>,
        original_state: Option<String>,
        original_sys_state: Option<FlowSysStateKind>,
        params: Vec<FlowExternalParams>,
        ctx: &TardisContext,
        funs: &TardisFunsInst,
    ) -> TardisResult<FlowExternalModifyFieldResp> {
        let external_url = Self::get_external_url(tag, ctx, funs).await?;
        if external_url.is_empty() {
            return Ok(FlowExternalModifyFieldResp {});
        }

        // complete changed_kind
        let params = params
            .into_iter()
            .map(|mut param| {
                if param.changed_kind.is_none() {
                    if param.value.clone().unwrap_or_default().to_string().is_empty() {
                        param.changed_kind = Some(FlowTransitionActionByVarChangeInfoChangedKind::Clean);
                    } else {
                        param.changed_kind = Some(FlowTransitionActionByVarChangeInfoChangedKind::ChangeContent);
                    }
                }
                param
            })
            .collect_vec();

        let header = Self::headers(None, funs, ctx).await?;
        let body = FlowExternalReq {
            kind: FlowExternalKind::ModifyField,
            callback_op,
            manual_op,
            operator,
            inst_id: inst_id.to_string(),
            curr_tag: tag.to_string(),
            curr_bus_obj_id: rel_business_obj_id.to_string(),
            target_state,
            target_sys_state,
            original_state,
            original_sys_state,
            notify: transition_detail.clone().map(|tran| tran.is_notify),
            transition_name: transition_detail.map(|tran| tran.name),
            sys_time: Some(Utc::now().timestamp_millis()),
            params,
            ..Default::default()
        };
        debug!("do_modify_field body: {:?}", body);
        let resp: FlowExternalResp<FlowExternalModifyFieldResp> = funs
            .web_client()
            .post(&external_url, &body, header)
            .await?
            .body
            .ok_or_else(|| funs.err().internal_error("flow_external", "do_modify_field", "illegal response", "500-external-illegal-response"))?;
        if resp.code != *"200" {
            return Err(funs.err().internal_error("flow_external", "do_modify_field", "illegal response", "500-external-illegal-response"));
        }
        if let Some(data) = resp.body {
            Ok(data)
        } else {
            Err(funs.err().internal_error("flow_external", "do_modify_field", "illegal response", "500-external-illegal-response"))
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn do_notify_changes(
        tag: &str,
        inst_id: &str,
        rel_business_obj_id: &str,
        target_state: String,
        target_sys_state: FlowSysStateKind,
        original_state: String,
        original_sys_state: FlowSysStateKind,
        transition_name: String,
        is_notify: bool,
        manual_op: Option<bool>,
        callback_op: Option<FlowExternalCallbackOp>,
        ctx: &TardisContext,
        funs: &TardisFunsInst,
    ) -> TardisResult<FlowExternalNotifyChangesResp> {
        let external_url = Self::get_external_url(tag, ctx, funs).await?;
        if external_url.is_empty() {
            return Ok(FlowExternalNotifyChangesResp {});
        }

        let header = Self::headers(None, funs, ctx).await?;
        let body = FlowExternalReq {
            kind: FlowExternalKind::NotifyChanges,
            callback_op,
            inst_id: inst_id.to_string(),
            curr_tag: tag.to_string(),
            curr_bus_obj_id: rel_business_obj_id.to_string(),
            target_state: Some(target_state),
            target_sys_state: Some(target_sys_state),
            original_state: Some(original_state),
            original_sys_state: Some(original_sys_state),
            transition_name: Some(transition_name),
            notify: Some(is_notify),
            manual_op,
            sys_time: Some(Utc::now().timestamp_millis()),
            ..Default::default()
        };
        debug!("do_notify_changes body: {:?}", body);
        let resp: FlowExternalResp<FlowExternalNotifyChangesResp> = funs
            .web_client()
            .post(&external_url, &body, header)
            .await?
            .body
            .ok_or_else(|| funs.err().internal_error("flow_external", "do_notify_changes", "illegal response", "500-external-illegal-response"))?;
        if resp.code != *"200" {
            return Err(funs.err().internal_error("flow_external", "do_notify_changes", "illegal response", "500-external-illegal-response"));
        }
        if let Some(data) = resp.body {
            Ok(data)
        } else {
            Err(funs.err().internal_error("flow_external", "do_notify_changes", "illegal response", "500-external-illegal-response"))
        }
    }

    pub async fn do_query_field(
        tag: &str,
        rel_business_obj_ids: Vec<String>,
        own_paths: &str,
        ctx: &TardisContext,
        funs: &TardisFunsInst,
    ) -> TardisResult<FlowExternalQueryFieldResp> {
        let external_url = Self::get_external_url(tag, ctx, funs).await?;
        if external_url.is_empty() {
            return Ok(FlowExternalQueryFieldResp::default());
        }

        let header = Self::headers(None, funs, ctx).await?;
        let body = FlowExternalReq {
            kind: FlowExternalKind::QueryField,
            inst_id: "".to_string(),
            curr_tag: tag.to_string(),
            curr_bus_obj_id: "".to_string(),
            owner_paths: own_paths.to_string(),
            obj_ids: rel_business_obj_ids,
            sys_time: Some(Utc::now().timestamp_millis()),
            ..Default::default()
        };
        debug!("do_query_field body: {:?}", body);
        let resp: FlowExternalResp<FlowExternalQueryFieldResp> = funs
            .web_client()
            .post(&external_url, &body, header)
            .await?
            .body
            .ok_or_else(|| funs.err().internal_error("flow_external", "do_query_field", "illegal response", "500-external-illegal-response"))?;
        if resp.code != *"200" {
            return Err(funs.err().internal_error("flow_external", "do_query_field", "illegal response", "500-external-illegal-response"));
        }
        if let Some(data) = resp.body {
            Ok(data)
        } else {
            Err(funs.err().internal_error("flow_external", "do_query_field", "illegal response", "500-external-illegal-response"))
        }
    }

    pub async fn do_delete_rel_obj(tag: &str, rel_business_obj_id: &str, inst_id: &str, ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<FlowExternalDeleteRelObjResp> {
        let external_url = Self::get_external_url(tag, ctx, funs).await?;
        if external_url.is_empty() {
            return Ok(FlowExternalDeleteRelObjResp {});
        }

        let header = Self::headers(None, funs, ctx).await?;
        let body = FlowExternalReq {
            kind: FlowExternalKind::DeleteObj,
            inst_id: inst_id.to_string(),
            curr_tag: tag.to_string(),
            curr_bus_obj_id: rel_business_obj_id.to_string(),
            sys_time: Some(Utc::now().timestamp_millis()),
            ..Default::default()
        };
        debug!("do_delete_rel_obj body: {:?}", body);
        let resp: FlowExternalResp<FlowExternalDeleteRelObjResp> = funs
            .web_client()
            .post(&external_url, &body, header)
            .await?
            .body
            .ok_or_else(|| funs.err().internal_error("flow_external", "do_delete_rel_obj", "illegal response", "500-external-illegal-response"))?;
        if resp.code != *"200" {
            return Err(funs.err().internal_error("flow_external", "do_delete_rel_obj", "illegal response", "500-external-illegal-response"));
        }
        if let Some(data) = resp.body {
            Ok(data)
        } else {
            Err(funs.err().internal_error("flow_external", "do_delete_rel_obj", "illegal response", "500-external-illegal-response"))
        }
    }

    async fn get_external_url(tag: &str, ctx: &TardisContext, funs: &TardisFunsInst) -> TardisResult<String> {
        let external_url = SpiKvClient::get_item(format!("{}:config:{}", flow_constants::DOMAIN_CODE, tag), None, funs, ctx)
            .await?
            .ok_or_else(|| funs.err().not_found("flow_external", "get_external_url", "not found external url", "404-external-data-url-not-exist"))?;
        Ok(external_url.value.as_str().unwrap_or_default().to_string())
    }

    async fn headers(headers: Option<Vec<(String, String)>>, _funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<(String, String)>> {
        let base_ctx = (TARDIS_CONTEXT.to_string(), TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(ctx)?));
        if let Some(mut headers) = headers {
            headers.push(base_ctx);
            return Ok(headers);
        }
        let headers = vec![base_ctx];
        Ok(headers)
    }
}
