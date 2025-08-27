use bios_basic::rbum::dto::rbum_filer_dto::RbumItemRelFilterReq;
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelModifyReq;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::{
    dto::{
        rbum_filer_dto::{RbumBasicFilterReq, RbumRelFilterReq},
        rbum_rel_agg_dto::{RbumRelAggAddReq, RbumRelEnvAggAddReq},
        rbum_rel_dto::{RbumRelAddReq, RbumRelBoneResp, RbumRelSimpleFindReq},
    },
    rbum_enumeration::{RbumRelEnvKind, RbumRelFromKind},
    serv::rbum_rel_serv::RbumRelServ,
};
use serde::{Deserialize, Serialize};

use strum::Display;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::{Duration, Utc},
    web::poem_openapi,
    TardisFunsInst,
};

pub struct FlowRelServ;

#[derive(poem_openapi::Enum, Display, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, strum::EnumString)]
pub enum FlowRelKind {
    FlowModelState,
    FlowModelTemplate,
    FlowAppTemplate,
    FlowModelTransition,
}

impl FlowRelServ {
    pub async fn add_simple_rel(
        flow_rel_kind: &FlowRelKind,
        from_rbum_id: &str,
        to_rbum_item_id: &str,
        start_timestamp: Option<i64>,
        end_timestamp: Option<i64>,
        ignore_exist_error: bool,
        to_is_outside: bool,
        ext: Option<String>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        if Self::exist_rels(flow_rel_kind, from_rbum_id, to_rbum_item_id, funs, ctx).await? {
            return if ignore_exist_error {
                Ok(())
            } else {
                Err(funs.err().conflict(&flow_rel_kind.to_string(), "add_simple_rel", "associated already exists", "409-rbum-rel-exist"))
            };
        }
        let value1 = start_timestamp.unwrap_or_else(|| Utc::now().timestamp());
        let value2 = end_timestamp.unwrap_or_else(|| (Utc::now() + Duration::try_days(365 * 100).expect("ignore")).timestamp());
        let req = &mut RbumRelAggAddReq {
            rel: RbumRelAddReq {
                tag: flow_rel_kind.to_string(),
                note: None,
                from_rbum_kind: RbumRelFromKind::Item,
                from_rbum_id: from_rbum_id.to_string(),
                to_rbum_item_id: to_rbum_item_id.to_string(),
                to_own_paths: ctx.own_paths.to_string(),
                to_is_outside,
                ext,
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

    pub async fn delete_simple_rel(flow_rel_kind: &FlowRelKind, from_rbum_id: &str, to_rbum_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let rel_ids = RbumRelServ::find_id_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: Some(flow_rel_kind.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(from_rbum_id.to_string()),
                to_rbum_item_id: Some(to_rbum_item_id.to_string()),
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
            RbumRelServ::delete_rel_with_ext(&rel_id, funs, ctx).await?;
        }

        Ok(())
    }

    pub async fn exist_rels(flow_rel_kind: &FlowRelKind, from_rbum_id: &str, to_rbum_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        RbumRelServ::check_simple_rel(
            &RbumRelSimpleFindReq {
                tag: Some(flow_rel_kind.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(from_rbum_id.to_string()),
                to_rbum_item_id: Some(to_rbum_item_id.to_string()),
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
        to_rbum_id: &str,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        let mock_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        RbumRelServ::find_to_simple_rels(&flow_rel_kind.to_string(), to_rbum_id, desc_sort_by_create, desc_sort_by_update, funs, &mock_ctx).await
    }

    pub async fn find_from_simple_rels(
        flow_rel_kind: &FlowRelKind,
        from_rbum_id: &str,
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
            from_rbum_id,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            &mock_ctx,
        )
        .await
    }

    pub async fn find_simple_rels(
        flow_rel_kind: &FlowRelKind,
        from_rbum_id: Option<&str>,
        to_rbum_item_id: Option<&str>,
        is_from: bool,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        let mock_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        let mut filter_req = RbumRelFilterReq {
            basic: RbumBasicFilterReq {
                with_sub_own_paths: true,
                ..Default::default()
            },
            tag: Some(flow_rel_kind.to_string()),
            ..Default::default()
        };
        if from_rbum_id.is_some() {
            filter_req.from_rbum_kind = Some(RbumRelFromKind::Item);
            filter_req.from_rbum_id = from_rbum_id.map(|s| s.to_string());
        }
        if to_rbum_item_id.is_some() {
            filter_req.to_rbum_item_id = to_rbum_item_id.map(|s| s.to_string());
        }

        RbumRelServ::find_simple_rels(&filter_req, desc_sort_by_create, desc_sort_by_update, is_from, funs, &mock_ctx).await
    }

    pub async fn modify_simple_rel(
        flow_rel_kind: &FlowRelKind,
        from_rbum_id: &str,
        to_rbum_item_id: &str,
        modify_req: &mut RbumRelModifyReq,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let rel_id = RbumRelServ::find_rel_ids(
            &RbumRelSimpleFindReq {
                tag: Some(flow_rel_kind.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(from_rbum_id.to_string()),
                to_rbum_item_id: Some(to_rbum_item_id.to_string()),
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

    pub fn get_template_rel_filter(rel_template_id: Option<&str>) -> Option<RbumItemRelFilterReq> {
        if rel_template_id.is_some() {
            Some(RbumItemRelFilterReq {
                optional: false,
                rel_by_from: true,
                tag: Some(FlowRelKind::FlowModelTemplate.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                rel_item_id: rel_template_id.map(|id| id.to_string()),
                ..Default::default()
            })
        } else {
            None
        }
    }

    pub async fn find_template_id_by_model_id(model_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<String>> {
        Ok(
            Self::find_from_simple_rels(&FlowRelKind::FlowModelTemplate, model_id, None, None, funs, ctx).await?.pop().map(|rel| rel.rel_id)
        )
    }
}
