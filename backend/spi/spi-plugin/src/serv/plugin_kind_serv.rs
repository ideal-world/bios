use bios_basic::{
    rbum::{
        dto::{
            rbum_filer_dto::{RbumBasicFilterReq, RbumKindFilterReq, RbumRelFilterReq},
            rbum_rel_dto::RbumRelBoneResp,
        },
        helper::rbum_scope_helper,
        rbum_enumeration::RbumRelFromKind,
        serv::{rbum_crud_serv::RbumCrudOperation, rbum_item_serv::RbumItemCrudOperation, rbum_kind_serv::RbumKindServ, rbum_rel_serv::RbumRelServ},
    },
    spi::{dto::spi_bs_dto::SpiBsFilterReq, serv::spi_bs_serv::SpiBsServ},
};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    TardisFunsInst,
};

use crate::{
    dto::plugin_kind_dto::{PluginKindAddAggReq, PluginKindAggResp},
    plugin_constants::KIND_MODULE_CODE,
    plugin_enumeration::PluginAppBindRelKind,
};

use super::{plugin_bs_serv::PluginBsServ, plugin_rel_serv::PluginRelServ};

pub struct PluginKindServ;

impl PluginKindServ {
    pub async fn add_kind_agg_rel(add_req: &PluginKindAddAggReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let bing_item_id = rbum_scope_helper::get_max_level_id_by_context(ctx).unwrap_or_default();
        let bs = SpiBsServ::peek_item(&add_req.bs_id, &SpiBsFilterReq::default(), funs, ctx).await?;
        if bs.kind_id != add_req.kind_id {
            return Err(funs.err().conflict("plugin_kind", "add_rel", "plugin bs kind mismatch", "409-spi-plugin-kind-mismatch"));
        }
        let rel_id = if let Some(mut bs_rel) = add_req.bs_rel.clone() {
            if bs_rel.rel_id.is_none() {
                bs_rel.rel_id = PluginBsServ::find_bs_rel_id(vec![bs_rel.bs_id.clone()], &bs_rel.app_tenant_id, funs, ctx).await?.first().cloned();
            }
            let rel_id = PluginBsServ::add_or_modify_plugin_rel_agg(&mut bs_rel.clone(), funs, ctx).await?;
            rel_id
        } else {
            if let Some(rel_id) = &add_req.rel_id {
                rel_id.clone()
            } else {
                return Err(funs.err().conflict("plugin_kind", "add_rel", "rel_id is required", "409-spi-plugin-required"));
            }
        };
        PluginRelServ::add_simple_rel(
            &PluginAppBindRelKind::PluginAppBindKind,
            &bing_item_id,
            &rel_id,
            None,
            Some(add_req.kind_id.clone()),
            add_req.ignore_exist.unwrap_or(false),
            true,
            add_req.attrs.clone(),
            funs,
            ctx,
        )
        .await?;
        Ok(())
    }

    pub async fn delete_kind_agg_rel(kind_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let bing_item_id = rbum_scope_helper::get_max_level_id_by_context(ctx).unwrap_or_default();
        for rel_bind in PluginRelServ::find_from_simple_rels(
            &PluginAppBindRelKind::PluginAppBindKind,
            &RbumRelFromKind::Item,
            &bing_item_id,
            Some(kind_id.to_string()),
            true,
            None,
            None,
            funs,
            ctx,
        )
        .await?
        {
            PluginRelServ::delete_simple_rel(&PluginAppBindRelKind::PluginAppBindKind, &bing_item_id, &rel_bind.rel_id, funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn delete_kind_agg_rel_by_rel_id(rel_id: &str, kind_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let bing_item_id = rbum_scope_helper::get_max_level_id_by_context(ctx).unwrap_or_default();
        let rel = PluginRelServ::get_rel(rel_id, funs, ctx).await?;
        if rel.from_rbum_id == bing_item_id && rel.ext == kind_id && rel.tag == PluginAppBindRelKind::PluginAppBindKind.to_string() {
            PluginRelServ::delete_simple_rel_by_id(rel_id, &PluginAppBindRelKind::PluginAppBindKind, &bing_item_id, rel_id, funs, ctx).await?;
            return Ok(());
        }
        Err(funs.err().not_found("plugin_kind", "delete_rel_by_rel_id", "not found rel", "404-spi-plugin-rel-not-exist"))
    }

    pub async fn find_kind_agg(
        parent_id: Option<String>,
        kind_codes: Option<Vec<String>>,
        app_tenant_id: &str,
        is_hide_secret: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<PluginKindAggResp>> {
        let kinds = RbumKindServ::find_detail_rbums(
            &RbumKindFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    codes: kind_codes,
                    ..Default::default()
                },
                module: Some(KIND_MODULE_CODE.to_string()),
                parent_id: parent_id,
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        let mut kind_aggs = Vec::new();
        for kind in kinds {
            let kind_agg = Self::get_kind_agg(&kind.id, app_tenant_id, is_hide_secret, funs, ctx).await?;
            if kind_agg.rel_bs.is_some() || kind_agg.rel_bind.is_some() {
                kind_aggs.push(kind_agg);
            }
        }
        Ok(kind_aggs)
    }

    pub async fn get_kind_agg(kind_id: &str, app_tenant_id: &str, is_hide_secret: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<PluginKindAggResp> {
        let kind = RbumKindServ::get_rbum(
            kind_id,
            &RbumKindFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                module: Some(KIND_MODULE_CODE.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        if let Some(rel_bind) = PluginRelServ::find_from_rels(
            &PluginAppBindRelKind::PluginAppBindKind,
            &RbumRelFromKind::Item,
            app_tenant_id,
            Some(kind_id.to_owned()),
            true,
            false,
            None,
            None,
            funs,
            ctx,
        )
        .await?
        .first()
        {
            match PluginRelServ::get_rel(&rel_bind.rel.to_rbum_item_id, funs, ctx).await {
                Ok(rel) => {
                    let mut rel_bs = PluginBsServ::get_bs(&rel.id, is_hide_secret, funs, ctx).await?;
                    if let Some(mut rel) = rel_bs.rel.clone() {
                        let mut new_attrs = rel_bind.attrs.clone();
                        rel.attrs.iter().for_each(|r| {
                            if rel_bind.attrs.iter().find(|e| e.name.eq_ignore_ascii_case(&r.name)).is_none() {
                                new_attrs.push(r.clone());
                            }
                        });
                        rel.attrs = new_attrs;
                        rel_bs.rel = Some(rel);
                    }
                    return Ok(PluginKindAggResp {
                        kind: kind.clone(),
                        rel_bind: Some(RbumRelBoneResp::new(rel_bind.rel.clone(), true)),
                        rel_bs: Some(rel_bs),
                    });
                }
                Err(_) => {
                    return Ok(PluginKindAggResp {
                        kind: kind.clone(),
                        rel_bind: None,
                        rel_bs: None,
                    });
                }
            }
        } else {
            return Ok(PluginKindAggResp {
                kind: kind.clone(),
                rel_bind: None,
                rel_bs: None,
            });
        }
    }

    pub async fn exist_kind_rel_by_kind_code(kind_code: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        if let Some(kind_id) = RbumKindServ::get_rbum_kind_id_by_code(kind_code, funs).await? {
            return Self::exist_kind_rel(&kind_id, funs, ctx).await;
        }
        Err(funs.err().not_found(&RbumKindServ::get_obj_name(), "get", "not found kind", "404-spi-plugin-kind-not-exist"))
    }

    pub async fn exist_kind_rel(kind_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        let bing_item_id = rbum_scope_helper::get_max_level_id_by_context(ctx).unwrap_or_default();
        RbumRelServ::exist_rbum(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: Some(PluginAppBindRelKind::PluginAppBindKind.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(bing_item_id.clone()),
                ext_eq: Some(kind_id.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await
    }
}
