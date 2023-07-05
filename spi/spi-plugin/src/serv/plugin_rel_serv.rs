use bios_basic::rbum::{
    dto::{
        rbum_filer_dto::{RbumBasicFilterReq, RbumRelExtFilterReq, RbumRelFilterReq},
        rbum_rel_agg_dto::{RbumRelAggAddReq, RbumRelAggResp},
        rbum_rel_dto::{RbumRelAddReq, RbumRelBoneResp, RbumRelDetailResp, RbumRelFindReq},
    },
    rbum_enumeration::RbumRelFromKind,
    serv::{
        rbum_crud_serv::RbumCrudOperation,
        rbum_rel_serv::{RbumRelAttrServ, RbumRelEnvServ, RbumRelServ},
    },
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    TardisFunsInst,
};

use crate::plugin_enumeration::PluginAppBindRelKind;
pub struct PluginRelServ;

impl PluginRelServ {
    pub async fn add_simple_rel(
        tag: &PluginAppBindRelKind,
        from_rbum_id: &str,
        to_rbum_item_id: &str,
        note: Option<String>,
        ext: Option<String>,
        ignore_exist_error: bool,
        to_is_outside: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        if Self::exist_rels(tag, from_rbum_id, to_rbum_item_id, funs, ctx).await? {
            return if ignore_exist_error {
                Ok(())
            } else {
                Err(funs.err().conflict(&tag.to_string(), "add_simple_rel", "associated already exists", "409-rbum-rel-exist"))
            };
        }
        let req = &mut RbumRelAggAddReq {
            rel: RbumRelAddReq {
                tag: tag.to_string(),
                note,
                from_rbum_kind: RbumRelFromKind::Item,
                from_rbum_id: from_rbum_id.to_string(),
                to_rbum_item_id: to_rbum_item_id.to_string(),
                to_own_paths: ctx.own_paths.to_string(),
                to_is_outside,
                ext,
            },
            attrs: vec![],
            envs: vec![],
        };
        RbumRelServ::add_rel(req, funs, ctx).await?;
        Ok(())
    }

    pub async fn delete_simple_rel(tag: &PluginAppBindRelKind, from_rbum_id: &str, to_rbum_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let rel_ids = RbumRelServ::find_id_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: Some(tag.to_string()),
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
            RbumRelServ::delete_rbum(&rel_id, funs, ctx).await?;
        }

        Ok(())
    }

    async fn exist_rels(tag: &PluginAppBindRelKind, from_rbum_id: &str, to_rbum_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        RbumRelServ::exist_simple_rel(
            &RbumRelFindReq {
                tag: Some(tag.to_string()),
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
        tag: &PluginAppBindRelKind,
        to_rbum_item_id: &str,
        ext: Option<String>,
        with: bool,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        RbumRelServ::find_simple_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: with,
                    ..Default::default()
                },
                tag: Some(tag.to_string()),
                to_rbum_item_id: Some(to_rbum_item_id.to_owned()),
                ext_eq: ext,
                ..Default::default()
            },
            desc_sort_by_create,
            desc_sort_by_update,
            false,
            funs,
            ctx,
        )
        .await
    }

    pub async fn find_from_simple_rels(
        tag: &PluginAppBindRelKind,
        from_rbum_kind: &RbumRelFromKind,
        from_rbum_id: &str,
        ext: Option<String>,
        with: bool,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        RbumRelServ::find_simple_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: with,
                    ..Default::default()
                },
                tag: Some(tag.to_string()),
                from_rbum_kind: Some(from_rbum_kind.clone()),
                from_rbum_id: Some(from_rbum_id.to_owned()),
                ext_eq: ext,
                ..Default::default()
            },
            desc_sort_by_create,
            desc_sort_by_update,
            true,
            funs,
            ctx,
        )
        .await
    }

    pub async fn get_rel(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumRelDetailResp> {
        let filter = RbumRelFilterReq {
            basic: RbumBasicFilterReq {
                with_sub_own_paths: true,
                own_paths: Some("".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let rel = RbumRelServ::get_rbum(id, &filter, funs, ctx).await?;
        Ok(rel)
    }

    pub async fn get_rel_agg(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumRelAggResp> {
        let filter = RbumRelFilterReq {
            basic: RbumBasicFilterReq {
                with_sub_own_paths: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let rbum_rel_id = id.to_string();
        let rel = Self::get_rel(id, funs, ctx).await?;
        let resp = RbumRelAggResp {
            rel,
            attrs: RbumRelAttrServ::find_rbums(
                &RbumRelExtFilterReq {
                    basic: filter.basic.clone(),
                    rel_rbum_rel_id: Some(rbum_rel_id.clone()),
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?,
            envs: RbumRelEnvServ::find_rbums(
                &RbumRelExtFilterReq {
                    basic: filter.basic.clone(),
                    rel_rbum_rel_id: Some(rbum_rel_id.clone()),
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?,
        };
        Ok(resp)
    }
}
