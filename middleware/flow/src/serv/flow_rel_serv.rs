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
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::{Duration, Utc},
    TardisFunsInst,
};

pub struct FlowRelServ;

const REL_KIND: &str = "FlowModelState";

impl FlowRelServ {
    pub async fn add_simple_rel(
        flow_model_id: &str,
        flow_state_id: &str,
        start_timestamp: Option<i64>,
        end_timestamp: Option<i64>,
        ignore_exist_error: bool,
        to_is_outside: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        if Self::exist_rels(flow_model_id, flow_state_id, funs, ctx).await? {
            return if ignore_exist_error {
                Ok(())
            } else {
                Err(funs.err().conflict(REL_KIND, "add_simple_rel", "associated already exists", "409-rbum-rel-exist"))
            };
        }
        let value1 = start_timestamp.unwrap_or_else(|| Utc::now().timestamp());
        let value2 = end_timestamp.unwrap_or_else(|| (Utc::now() + Duration::days(365 * 100)).timestamp());
        let req = &mut RbumRelAggAddReq {
            rel: RbumRelAddReq {
                tag: REL_KIND.to_string(),
                note: None,
                from_rbum_kind: RbumRelFromKind::Item,
                from_rbum_id: flow_model_id.to_string(),
                to_rbum_item_id: flow_state_id.to_string(),
                to_own_paths: ctx.own_paths.to_string(),
                to_is_outside,
                ext: None,
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

    pub async fn delete_simple_rel(flow_model_id: &str, flow_state_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let rel_ids = RbumRelServ::find_id_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: Some(REL_KIND.to_string()),
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

    async fn exist_rels(flow_model_id: &str, flow_state_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        // TODO In-depth inspection
        RbumRelServ::exist_simple_rel(
            &RbumRelFindReq {
                tag: Some(REL_KIND.to_string()),
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
        flow_model_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        RbumRelServ::find_to_simple_rels(REL_KIND, flow_model_id, desc_sort_by_create, desc_sort_by_update, funs, ctx).await
    }
}
