use bios_basic::rbum::dto::rbum_rel_dto::RbumRelModifyReq;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::{
    dto::{
        rbum_filer_dto::{RbumBasicFilterReq, RbumRelFilterReq},
        rbum_rel_agg_dto::{RbumRelAggAddReq, RbumRelEnvAggAddReq},
        rbum_rel_dto::{RbumRelAddReq, RbumRelBoneResp, RbumRelFindReq},
    },
    rbum_enumeration::{RbumRelEnvKind, RbumRelFromKind},
    serv::rbum_rel_serv::RbumRelServ,
};
use serde::{Deserialize, Serialize};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::{Duration, Utc},
    TardisFunsInst,
};

pub struct FlowRelServ;

#[derive(strum::Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, strum::EnumString)]
pub enum FlowRelKind {
    FlowModelState,
}

impl FlowRelServ {
    pub async fn add_simple_rel(
        flow_rel_kind: &FlowRelKind,
        flow_model_id: &str,
        flow_state_id: &str,
        start_timestamp: Option<i64>,
        end_timestamp: Option<i64>,
        ignore_exist_error: bool,
        to_is_outside: bool,
        sort: Option<i64>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        if Self::exist_rels(flow_rel_kind, flow_model_id, flow_state_id, funs, ctx).await? {
            return if ignore_exist_error {
                Ok(())
            } else {
                Err(funs.err().conflict(&flow_rel_kind.to_string(), "add_simple_rel", "associated already exists", "409-rbum-rel-exist"))
            };
        }
        let value1 = start_timestamp.unwrap_or_else(|| Utc::now().timestamp());
        let value2 = end_timestamp.unwrap_or_else(|| (Utc::now() + Duration::days(365 * 100)).timestamp());
        let req = &mut RbumRelAggAddReq {
            rel: RbumRelAddReq {
                tag: flow_rel_kind.to_string(),
                note: None,
                from_rbum_kind: RbumRelFromKind::Item,
                from_rbum_id: flow_model_id.to_string(),
                to_rbum_item_id: flow_state_id.to_string(),
                to_own_paths: ctx.own_paths.to_string(),
                to_is_outside,
                ext: sort.map(|sort| sort.to_string()),
            },
            attrs: vec![],
            envs: if start_timestamp.is_some() || end_timestamp.is_some() {
                vec![RbumRelEnvAggAddReq {
                    kind: RbumRelEnvKind::DatetimeRange,
                    value1: value1.to_string(),
                    value2: Some(value2.to_string()),
                }]
            } else {
                vec![]
            },
        };
        RbumRelServ::add_rel(req, funs, ctx).await?;
        Ok(())
    }

    pub async fn delete_simple_rel(flow_rel_kind: &FlowRelKind, flow_model_id: &str, flow_state_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let rel_ids = RbumRelServ::find_id_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: Some(flow_rel_kind.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(flow_model_id.to_string()),
                to_rbum_item_id: Some(flow_state_id.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        if rel_ids.is_empty() {
            return Ok(());
        }
        for rel_id in rel_ids {
            RbumRelServ::delete_rbum(&rel_id, funs, ctx).await?;
        }

        Ok(())
    }

    async fn exist_rels(flow_rel_kind: &FlowRelKind, flow_model_id: &str, flow_state_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        // TODO In-depth inspection
        RbumRelServ::exist_simple_rel(
            &RbumRelFindReq {
                tag: Some(flow_rel_kind.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(flow_model_id.to_string()),
                to_rbum_item_id: Some(flow_state_id.to_string()),
                from_own_paths: Some(ctx.own_paths.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn find_to_simple_rels(
        flow_rel_kind: &FlowRelKind,
        flow_model_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        let mock_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        RbumRelServ::find_to_simple_rels(&flow_rel_kind.to_string(), flow_model_id, desc_sort_by_create, desc_sort_by_update, funs, &mock_ctx).await
    }

    pub async fn find_from_simple_rels(
        flow_rel_kind: &FlowRelKind,
        flow_model_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        let mock_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        RbumRelServ::find_from_simple_rels(
            &flow_rel_kind.to_string(),
            &RbumRelFromKind::Item,
            true,
            flow_model_id,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            &mock_ctx,
        )
        .await
    }

    pub async fn modify_simple_rel(
        flow_rel_kind: &FlowRelKind,
        flow_model_id: &str,
        flow_state_id: &str,
        modify_req: &mut RbumRelModifyReq,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let rel_id = RbumRelServ::find_rel_ids(
            &RbumRelFindReq {
                tag: Some(flow_rel_kind.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(flow_model_id.to_string()),
                to_rbum_item_id: Some(flow_state_id.to_string()),
                from_own_paths: None,
                to_rbum_own_paths: None,
            },
            funs,
            ctx,
        )
        .await?
        .pop();
        if let Some(rel_id) = rel_id {
            RbumRelServ::modify_rbum(&rel_id, modify_req, funs, ctx).await?;
        } else {
            return Err(funs.err().conflict(&flow_rel_kind.to_string(), "modify_simple_rel", "rel not found", "404-rel-not-found"));
        }
        Ok(())
    }
}
