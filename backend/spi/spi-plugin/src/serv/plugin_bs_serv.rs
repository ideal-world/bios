use bios_basic::{
    rbum::{
        dto::{
            rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq, RbumKindFilterReq, RbumRelFilterReq},
            rbum_rel_agg_dto::RbumRelAggModifyReq,
            rbum_rel_dto::RbumRelModifyReq,
        },
        helper::rbum_scope_helper,
        rbum_enumeration::RbumRelFromKind,
        serv::{rbum_crud_serv::RbumCrudOperation, rbum_item_serv::RbumItemCrudOperation, rbum_kind_serv::RbumKindServ, rbum_rel_serv::RbumRelServ},
    },
    spi::{
        dto::spi_bs_dto::SpiBsFilterReq,
        serv::spi_bs_serv::SpiBsServ,
        spi_constants::{self},
    },
};
use bios_sdk_invoke::clients::{
    spi_kv_client::SpiKvClient,
    spi_log_client::{LogDynamicContentReq, SpiLogClient},
};
use serde::{Deserialize, Serialize};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log::info,
    tokio,
    web::web_resp::TardisPage,
    TardisFunsInst,
};

use crate::{
    dto::plugin_bs_dto::{PluginBsAddReq, PluginBsCertInfoResp, PluginBsInfoResp},
    plugin_config::PluginConfig,
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
            let rel_name_clone = add_req.name.clone();
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
                        let _ = SpiKvClient::add_or_modify_key_name(
                            &format!("{}{}", &funs.conf::<PluginConfig>().kv_plugin_prefix.clone(), rel_id_clone),
                            &rel_name_clone,
                            None,
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
            RbumRelServ::modify_rel(
                &rel_id,
                &mut RbumRelAggModifyReq {
                    rel: RbumRelModifyReq {
                        tag: None,
                        note: Some(add_req.name.clone()),
                        ext: None,
                        disabled: None,
                    },
                    attrs: add_req.attrs.clone().unwrap_or(vec![]),
                    envs: vec![],
                },
                funs,
                ctx,
            )
            .await?;
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
            let rel_name_clone = add_req.name.clone();
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
                            Some("新增".to_string()),
                            None,
                            Some(tardis::chrono::Utc::now().to_rfc3339()),
                            &funs,
                            &ctx_clone,
                        )
                        .await;
                        let _ = SpiKvClient::add_or_modify_key_name(
                            &format!("{}{}", &funs.conf::<PluginConfig>().kv_plugin_prefix.clone(), rel_id_clone),
                            &rel_name_clone,
                            None,
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
        let rel_agg = PluginRelServ::get_rel_agg(rel_id, false, funs, ctx).await?;
        if PluginRelServ::exist_to_simple_rels(&PluginAppBindRelKind::PluginAppBindKind, &rel_agg.rel.id, funs, ctx).await? {
            return Err(funs.err().unauthorized("spi_bs", "delete_plugin_rel", "The pluging exists bound", "409-spi-plugin-bind-exist"));
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
        #[cfg(feature = "with-mq")]
        {
            use std::collections::HashMap;
            if funs.conf::<PluginConfig>().use_mq {
                use tardis::serde_json::json;

                funs.mq()
                    .publish(
                        &funs.conf::<PluginConfig>().mq_topic_event_plugin_delete,
                        json!({ "rel_id": rel_id.to_string() }).to_string(),
                        &HashMap::new(),
                    )
                    .await?;
            }
        }
        SpiBsServ::disable_rel(rel_id, funs, ctx).await?;
        Ok(())
    }

    pub async fn find_sub_bind_ids_bak(rel_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        let rel_agg = PluginRelServ::get_rel_agg(rel_id, false, funs, ctx).await?;
        let app_ids = PluginRelServ::find_to_simple_rels(&PluginAppBindRelKind::PluginAppBindKind, &rel_agg.rel.id, None, true, None, None, funs, ctx)
            .await?
            .into_iter()
            .map(|resp| resp.rel_id)
            .collect::<Vec<String>>();
        Ok(app_ids)
    }

    // todo remove
    pub async fn find_sub_bind_ids(bs_id: &str, app_tenant_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        let rels = RbumRelServ::find_detail_rbums(
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
        let mut rel_ids = vec![];
        for rel in rels {
            let app_ids = PluginRelServ::find_to_simple_rels(&PluginAppBindRelKind::PluginAppBindKind, &rel.id, None, true, None, None, funs, ctx)
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
        bs_id: Option<String>,
        app_tenant_id: &str,
        is_hide_secret: bool,
        page_number: u32,
        page_size: u32,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<PluginBsInfoResp>> {
        let bs_ids = if bs_id.is_some() {
            Some(vec![bs_id.clone().unwrap()])
        } else {
            if let Some(kind_id) = kind_id {
                let bs_ids = SpiBsServ::find_id_items(
                    &SpiBsFilterReq {
                        basic: RbumBasicFilterReq {
                            with_sub_own_paths: true,
                            own_paths: Some("".to_string()),
                            ..Default::default()
                        },
                        kind_id: Some(kind_id),
                        ..Default::default()
                    },
                    None,
                    None,
                    funs,
                    ctx,
                )
                .await?;
                if bs_ids.len() == 0 {
                    return Ok(TardisPage {
                        page_size: page_size as u64,
                        page_number: page_number as u64,
                        total_size: 0,
                        records: vec![],
                    });
                }
                Some(bs_ids)
            } else {
                None
            }
        };
        let rel_agg = PluginRelServ::paginate_rels(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    own_paths: Some("".to_string()),
                    ..Default::default()
                },
                tag: Some(spi_constants::SPI_IDENT_REL_TAG.to_owned()),
                from_rbum_ids: bs_ids,
                to_rbum_item_id: Some(app_tenant_id.to_string()),
                disabled: Some(false),
                ..Default::default()
            },
            is_hide_secret,
            page_number,
            page_size,
            desc_by_create,
            desc_by_update,
            funs,
            ctx,
        )
        .await?;
        let mut bs_records = vec![];
        for rel_agg in rel_agg.records {
            let bs = SpiBsServ::peek_item(&rel_agg.rel.from_rbum_id, &SpiBsFilterReq::default(), funs, ctx).await?;
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

    pub async fn exist_bs(bs_id: &str, app_tenant_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        if SpiBsServ::count_items(
            &SpiBsFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(vec![bs_id.to_string()]),
                    ..Default::default()
                },
                rel: Some(RbumItemRelFilterReq {
                    rel_by_from: true,
                    tag: Some(spi_constants::SPI_IDENT_REL_TAG.to_string()),
                    from_rbum_kind: Some(RbumRelFromKind::Item),
                    rel_item_id: Some(app_tenant_id.to_owned()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
            > 0
        {
            return Ok(true);
        }
        Ok(false)
    }

    pub async fn get_bs(rel_id: &str, is_hide_secret: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<PluginBsInfoResp> {
        let rel_agg = PluginRelServ::get_rel_agg(rel_id, is_hide_secret, funs, ctx).await?;
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

    pub async fn get_cert_bs(rel_id: &str, is_hide_secret: bool, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<PluginBsCertInfoResp> {
        let rel_agg = PluginRelServ::get_rel_agg(rel_id, is_hide_secret, funs, ctx).await?;
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

    pub async fn get_bs_by_up_kind_code(
        kind_code: &str,
        rel_id: Option<String>,
        is_hide_secret: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<PluginBsCertInfoResp> {
        let kind_id = RbumKindServ::get_rbum_kind_id_by_code(kind_code, funs).await?;
        if let Some(rel_id) = rel_id {
            // use can specify the rel_id to get the specific plugin
            if !RbumRelServ::exist_rbum(
                &RbumRelFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        own_paths: Some("".to_string()),
                        ids: Some(vec![rel_id.clone()]),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            {
                return Err(funs.err().not_found(&SpiBsServ::get_obj_name(), "get_bs_by_rel_up", "not found backend service", "404-spi-bs-not-exist"));
            }
            let rel = PluginRelServ::get_rel(&rel_id, funs, ctx).await?;
            return Self::get_cert_bs(&rel.id, is_hide_secret, funs, ctx).await;
        } else {
            if let Some(kind_id) = kind_id {
                // find the first binding relationship by kind_code
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
                    return Self::get_cert_bs(&rel.id, is_hide_secret, funs, ctx).await;
                } else {
                    // up own_paths
                    let bs_ids = SpiBsServ::find_id_items(
                        &SpiBsFilterReq {
                            basic: RbumBasicFilterReq {
                                with_sub_own_paths: true,
                                own_paths: Some("".to_string()),
                                ..Default::default()
                            },
                            kind_id: Some(kind_id.to_string()),
                            ..Default::default()
                        },
                        None,
                        None,
                        funs,
                        ctx,
                    )
                    .await?;
                    let own_paths = Self::get_parent_own_paths(ctx.own_paths.as_str())?;
                    for app_tenant_id in own_paths {
                        let rel_ids = Self::find_bs_rel_id(bs_ids.clone(), app_tenant_id.as_str(), funs, ctx).await?;
                        info!("【get_bs_by_rel_up】 {}: {}", app_tenant_id, rel_ids.len());
                        if rel_ids.len() > 0 {
                            match rel_ids.first() {
                                Some(rel_id) => {
                                    return Self::get_cert_bs(rel_id, is_hide_secret, funs, ctx).await;
                                }
                                None => return Err(funs.err().not_found(&SpiBsServ::get_obj_name(), "get_bs_by_rel_up", "not found backend service", "404-spi-bs-not-exist")),
                            }
                        }
                    }
                    return Err(funs.err().not_found(&SpiBsServ::get_obj_name(), "get_bs_by_rel_up", "not found backend service", "404-spi-bs-not-exist"));
                }
            }
        }
        Err(funs.err().not_found(&SpiBsServ::get_obj_name(), "get_bs_by_rel_up", "not found backend service", "404-spi-bs-not-exist"))
    }

    pub async fn find_bs_rel_id(bs_ids: Vec<String>, app_tenant_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        let rel_ids = RbumRelServ::find_id_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                tag: Some(spi_constants::SPI_IDENT_REL_TAG.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Item),
                from_rbum_ids: Some(bs_ids),
                to_rbum_item_id: Some(app_tenant_id.to_owned()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        Ok(rel_ids)
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
