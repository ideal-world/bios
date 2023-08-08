use bios_basic::{
    rbum::{
        dto::rbum_filer_dto::{RbumBasicFilterReq, RbumKindFilterReq, RbumRelFilterReq},
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
            return Err(funs.err().conflict("plugin_kind", "add_rel", "plugin bs kind mismatch", ""));
        }
        if let Some(bs_rel) = add_req.bs_rel.clone() {
            PluginBsServ::add_or_modify_plugin_rel_agg(&add_req.bs_id, &add_req.app_tenant_id, &mut bs_rel.clone(), funs, ctx).await?;
        }
        let bs_rel = PluginBsServ::get_bs_rel_agg(&add_req.bs_id, &add_req.app_tenant_id, funs, ctx).await?;
        // if RbumRelServ::count_rbums(
        //     &RbumRelFilterReq {
        //         basic: RbumBasicFilterReq {
        //             with_sub_own_paths: true,
        //             ..Default::default()
        //         },
        //         tag: Some(PluginAppBindRelKind::PluginAppBindKind.to_string()),
        //         from_rbum_kind: Some(RbumRelFromKind::Item),
        //         from_rbum_id: Some(bing_item_id.clone()),
        //         ext_eq: Some(add_req.kind_id.clone()),
        //         ..Default::default()
        //     },
        //     funs,
        //     ctx,
        // )
        // .await?
        //     == 0
        // {
        Self::delete_kind_agg_rel(&add_req.kind_id, funs, ctx).await?;
        PluginRelServ::add_simple_rel(
            &PluginAppBindRelKind::PluginAppBindKind,
            &bing_item_id,
            &bs_rel.rel.id,
            None,
            Some(add_req.kind_id.clone()),
            false,
            true,
            funs,
            ctx,
        )
        .await?;
        // }
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

    pub async fn find_kind_agg(app_tenant_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<PluginKindAggResp>> {
        let kinds = RbumKindServ::find_detail_rbums(
            &RbumKindFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                module: Some(KIND_MODULE_CODE.to_string()),
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        let mut kind_aggs = Vec::new();
        for kind in kinds {
            if let Some(rel_bind) = PluginRelServ::find_from_simple_rels(
                &PluginAppBindRelKind::PluginAppBindKind,
                &RbumRelFromKind::Item,
                app_tenant_id,
                Some(kind.id.clone()),
                true,
                None,
                None,
                funs,
                ctx,
            )
            .await?
            .get(0)
            {
                match PluginRelServ::get_rel(&rel_bind.rel_id, funs, ctx).await {
                    Ok(rel) => {
                        let rel_bs = PluginBsServ::get_bs(&rel.from_rbum_id, &rel.to_rbum_item_id, funs, ctx).await?;
                        kind_aggs.push(PluginKindAggResp {
                            kind: kind.clone(),
                            rel_bind: Some(rel_bind.clone()),
                            rel_bs: Some(rel_bs),
                        });
                    }
                    Err(_) => {
                        kind_aggs.push(PluginKindAggResp {
                            kind: kind.clone(),
                            rel_bind: None,
                            rel_bs: None,
                        });
                    }
                }
            } else {
                kind_aggs.push(PluginKindAggResp {
                    kind: kind.clone(),
                    rel_bind: None,
                    rel_bs: None,
                });
            }
        }
        Ok(kind_aggs)
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
