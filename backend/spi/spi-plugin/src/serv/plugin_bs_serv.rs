use bios_basic::{
    rbum::{
        dto::{
            rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq, RbumKindFilterReq, RbumRelFilterReq},
            rbum_rel_agg_dto::RbumRelAggResp,
            rbum_rel_attr_dto::RbumRelAttrAddReq,
            rbum_rel_dto::RbumRelModifyReq,
        },
        helper::rbum_scope_helper,
        rbum_enumeration::RbumRelFromKind,
        serv::{
            rbum_crud_serv::RbumCrudOperation,
            rbum_item_serv::RbumItemCrudOperation,
            rbum_kind_serv::RbumKindServ,
            rbum_rel_serv::{RbumRelAttrServ, RbumRelServ},
        },
    },
    spi::{
        dto::spi_bs_dto::SpiBsFilterReq,
        serv::spi_bs_serv::SpiBsServ,
        spi_constants::{self},
    },
};
use bios_sdk_invoke::clients::spi_log_client::{LogDynamicContentReq, SpiLogClient};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log::info,
    tokio,
    web::web_resp::TardisPage,
    TardisFunsInst,
};

use crate::{
    dto::plugin_bs_dto::{PluginBsAddReq, PluginBsCertInfoResp, PluginBsInfoResp},
    plugin_enumeration::PluginAppBindRelKind,
};

use super::plugin_rel_serv::PluginRelServ;

pub struct PluginBsServ;

impl PluginBsServ {
    pub async fn add_or_modify_plugin_rel_agg(add_req: &mut PluginBsAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let ctx_clone = ctx.clone();
        if !ctx.own_paths.contains(&add_req.app_tenant_id) {
            return Err(funs.err().unauthorized(
                "spi_bs",
                "add_or_modify_plugin_rel_agg",
                &format!("plugin binding rel unauthorized {}.{} by {}", add_req.bs_id, add_req.app_tenant_id, ctx.own_paths),
                "401-plugin-ownership-illegal",
            ));
        }
        let bs = SpiBsServ::peek_item(&add_req.bs_id, &SpiBsFilterReq::default(), funs, ctx).await?;
        let rel_id = if let Some(rel_id) = add_req.clone().rel_id {
            let rel_id_clone = rel_id.clone();
            ctx.add_async_task(Box::new(|| {
                Box::pin(async move {
                    let task_handle = tokio::spawn(async move {
                        let funs = crate::get_tardis_inst();
                        let _ = SpiLogClient::add_dynamic_log(
                            &LogDynamicContentReq {
                                details: None,
                                sub_kind: None,
                                content: Some(format!("插件 {}", bs.name)),
                            },
                            None,
                            Some("dynamic_log_plugin_manage".to_string()),
                            Some(rel_id_clone.clone()),
                            Some("编辑".to_string()),
                            None,
                            Some(tardis::chrono::Utc::now().to_rfc3339()),
                            &funs,
                            &ctx_clone,
                        )
                        .await;
                    });
                    let _ = task_handle.await;
                    Ok(())
                })
            }))
            .await?;
            let rel_agg = Self::get_bs_rel_agg(&rel_id, funs, ctx).await?;
            RbumRelServ::modify_rbum(
                &rel_id,
                &mut RbumRelModifyReq {
                    tag: None,
                    note: Some(add_req.name.clone()),
                    ext: None,
                },
                funs,
                ctx,
            )
            .await?;
            for attrs in rel_agg.attrs {
                RbumRelAttrServ::delete_rbum(&attrs.id, funs, ctx).await?;
            }
            if let Some(attrs) = &add_req.attrs {
                // TODO check attrs
                for attr in attrs {
                    RbumRelAttrServ::add_rbum(
                        &mut RbumRelAttrAddReq {
                            is_from: attr.is_from,
                            value: attr.value.to_string(),
                            name: attr.name.clone(),
                            rel_rbum_rel_id: rel_agg.rel.id.to_string(),
                            rel_rbum_kind_attr_id: attr.rel_rbum_kind_attr_id.clone(),
                            record_only: attr.record_only,
                        },
                        funs,
                        ctx,
                    )
                    .await?;
                }
            }
            rel_id
        } else {
            let rel_id = SpiBsServ::add_rel_agg(
                bs.id.as_str(),
                &add_req.clone().app_tenant_id,
                false,
                Some(add_req.name.clone()),
                add_req.clone().attrs,
                None,
                funs,
                ctx,
            )
            .await?;
            let rel_id_clone = rel_id.clone();
            ctx.add_async_task(Box::new(|| {
                Box::pin(async move {
                    let task_handle = tokio::spawn(async move {
                        let funs = crate::get_tardis_inst();
                        let _ = SpiLogClient::add_dynamic_log(
                            &LogDynamicContentReq {
                                details: None,
                                sub_kind: None,
                                content: Some(format!("插件 {}", bs.name)),
                            },
                            None,
                            Some("dynamic_log_plugin_manage".to_string()),
                            Some(rel_id_clone),
                            Some("新增".to_string()),
                            None,
                            Some(tardis::chrono::Utc::now().to_rfc3339()),
                            &funs,
                            &ctx_clone,
                        )
                        .await;
                    });
                    let _ = task_handle.await;
                    Ok(())
                })
            }))
            .await?;
            rel_id
        };
        Ok(rel_id)
    }

    pub async fn delete_plugin_rel(rel_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        let rel_agg = Self::get_bs_rel_agg(rel_id, funs, ctx).await?;
        if PluginRelServ::exist_to_simple_rels(&PluginAppBindRelKind::PluginAppBindKind, &rel_agg.rel.id, funs, ctx).await? {
            return Err(funs.err().unauthorized("spi_bs", "delete_plugin_rel", "The pluging exists bound", "401-spi-plugin-bind-exist"));
        }
        let bs = SpiBsServ::peek_item(&rel_agg.rel.from_rbum_id, &SpiBsFilterReq::default(), funs, ctx).await?;
        let _ = SpiLogClient::add_dynamic_log(
            &LogDynamicContentReq {
                details: None,
                sub_kind: None,
                content: Some(format!("插件 {}", bs.name)),
            },
            None,
            Some("dynamic_log_plugin_manage".to_string()),
            Some(rel_id.to_string()),
            Some("删除".to_string()),
            None,
            Some(tardis::chrono::Utc::now().to_rfc3339()),
            funs,
            &ctx_clone,
        )
        .await;
        SpiBsServ::delete_rel(rel_id, funs, ctx).await?;
        Ok(())
    }

    pub async fn find_sub_bind_ids_bak(rel_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        let rel_agg = Self::get_bs_rel_agg(rel_id, funs, ctx).await?;
        let app_ids = PluginRelServ::find_to_simple_rels(&PluginAppBindRelKind::PluginAppBindKind, &rel_agg.rel.id, None, true, None, None, funs, ctx)
            .await?
            .into_iter()
            .map(|resp| resp.rel_id)
            .collect::<Vec<String>>();
        Ok(app_ids)
    }

    // todo remove
    pub async fn find_sub_bind_ids(bs_id: &str, app_tenant_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        let rel_aggs = Self::find_bs_rel_agg(bs_id, app_tenant_id, funs, ctx).await?;
        let mut rel_ids = vec![];
        for rel_agg in rel_aggs {
            let app_ids = PluginRelServ::find_to_simple_rels(&PluginAppBindRelKind::PluginAppBindKind, &rel_agg.rel.id, None, true, None, None, funs, ctx)
                .await?
                .into_iter()
                .map(|resp| resp.rel_id)
                .collect::<Vec<String>>();
            rel_ids.extend(app_ids);
        }
        Ok(rel_ids)
    }

    pub async fn paginate_bs_rel_agg(
        kind_id: Option<String>,
        app_tenant_id: &str,
        page_number: u32,
        page_size: u32,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<PluginBsInfoResp>> {
        let rel_agg = RbumRelServ::paginate_to_rels(
            spi_constants::SPI_IDENT_REL_TAG,
            app_tenant_id,
            page_number,
            page_size,
            Some("".to_string()),
            desc_by_create,
            desc_by_update,
            funs,
            ctx,
        )
        .await?;
        let mut bs_records = vec![];
        for rel_agg in rel_agg.records {
            let bs = SpiBsServ::peek_item(&rel_agg.rel.from_rbum_id, &SpiBsFilterReq::default(), funs, ctx).await?;
            if let Some(kind_id) = kind_id.clone() {
                if bs.kind_id != kind_id {
                    continue;
                }
            }
            let kind = RbumKindServ::get_rbum(
                &bs.kind_id,
                &RbumKindFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        own_paths: Some("".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            bs_records.push(PluginBsInfoResp {
                id: bs.id,
                name: bs.name,
                kind_id: bs.kind_id,
                kind_code: bs.kind_code,
                kind_name: bs.kind_name,
                kind_parent_id: Some(kind.parent_id.clone()),
                kind_parent_name: kind.parent_name.clone(),
                rel: Some(rel_agg),
            });
        }
        Ok(TardisPage {
            page_size: page_size as u64,
            page_number: page_number as u64,
            total_size: rel_agg.total_size,
            records: bs_records,
        })
    }

    // todo
    // pub async fn exist_bs(bs_id: &str, app_tenant_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    //     if SpiBsServ::count_items(
    //         &SpiBsFilterReq {
    //             basic: RbumBasicFilterReq {
    //                 ids: Some(vec![bs_id.to_string()]),
    //                 ..Default::default()
    //             },
    //             rel: Some(RbumItemRelFilterReq {
    //                 rel_by_from: true,
    //                 tag: Some(spi_constants::SPI_IDENT_REL_TAG.to_string()),
    //                 from_rbum_kind: Some(RbumRelFromKind::Item),
    //                 rel_item_id: Some(app_tenant_id.to_owned()),
    //                 ..Default::default()
    //             }),
    //             ..Default::default()
    //         },
    //         funs,
    //         ctx,
    //     )
    //     .await?
    //         > 0
    //     {
    //         return Ok(true);
    //     }
    //     Ok(false)
    // }

    pub async fn get_bs(rel_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<PluginBsInfoResp> {
        let rel_agg = Self::get_bs_rel_agg(rel_id, funs, ctx).await?;
        let bs = SpiBsServ::peek_item(
            &rel_agg.rel.from_rbum_id,
            &SpiBsFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let kind = RbumKindServ::get_rbum(
            &bs.kind_id,
            &RbumKindFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        Ok(PluginBsInfoResp {
            id: bs.id,
            name: bs.name,
            kind_id: bs.kind_id,
            kind_code: bs.kind_code,
            kind_name: bs.kind_name,
            kind_parent_id: Some(kind.parent_id.clone()),
            kind_parent_name: kind.parent_name.clone(),
            rel: Some(rel_agg),
        })
    }

    pub async fn get_cert_bs(rel_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<PluginBsCertInfoResp> {
        let rel_agg = Self::get_bs_rel_agg(rel_id, funs, ctx).await?;
        let bs = SpiBsServ::peek_item(&rel_agg.rel.from_rbum_id, &SpiBsFilterReq::default(), funs, ctx).await?;
        Ok(PluginBsCertInfoResp {
            id: bs.id,
            name: bs.name,
            conn_uri: bs.conn_uri,
            ak: bs.ak,
            sk: bs.sk,
            ext: bs.ext,
            private: bs.private,
            rel: Some(rel_agg),
        })
    }

    pub async fn get_bs_by_kind_code(kind_code: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<PluginBsCertInfoResp> {
        let kind_id = RbumKindServ::get_rbum_kind_id_by_code(kind_code, funs).await?;
        if let Some(kind_id) = kind_id {
            if let Some(rel_bind) = PluginRelServ::find_from_simple_rels(
                &PluginAppBindRelKind::PluginAppBindKind,
                &RbumRelFromKind::Item,
                &rbum_scope_helper::get_max_level_id_by_context(ctx).unwrap_or_default(),
                Some(kind_id.clone()),
                true,
                None,
                None,
                funs,
                ctx,
            )
            .await?
            .first()
            {
                let rel = PluginRelServ::get_rel(&rel_bind.rel_id, funs, ctx).await?;
                return Self::get_cert_bs(&rel.id, funs, ctx).await;
            } else {
                return Err(funs.err().not_found(&SpiBsServ::get_obj_name(), "get_bs_by_rel_up", "not found backend service", "404-spi-bs-not-exist"));
            }
        }
        Err(funs.err().not_found(&SpiBsServ::get_obj_name(), "get_bs_by_rel_up", "not found backend service", "404-spi-bs-not-exist"))
    }

    pub async fn find_bs_rel_agg(bs_id: &str, app_tenant_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<RbumRelAggResp>> {
        let rel_agg = RbumRelServ::find_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: Some(spi_constants::SPI_IDENT_REL_TAG.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_id: Some(bs_id.to_owned()),
                to_rbum_item_id: Some(app_tenant_id.to_owned()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        Ok(rel_agg)
    }

    pub async fn get_bs_rel_agg(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<RbumRelAggResp> {
        let mut rel_agg = RbumRelServ::find_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ids: Some(vec![id.to_string()]),
                    ..Default::default()
                },
                tag: Some(spi_constants::SPI_IDENT_REL_TAG.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        if let Some(last) = rel_agg.pop() {
            // it means it's unique
            if rel_agg.is_empty() {
                return Ok(last);
            }
        }
        Err(funs.err().conflict(&SpiBsServ::get_obj_name(), "get_bs", "Not Configured bs app_tenant_id .", ""))
    }

    pub fn get_parent_own_paths(own_paths: &str) -> TardisResult<Vec<String>> {
        if own_paths.is_empty() {
            return Ok(vec!["".to_string()]);
        }
        let mut paths = own_paths.split('/').map(|s| s.to_string()).collect::<Vec<String>>();
        paths.reverse();
        paths.push("".to_string());
        Ok(paths)
    }
}
