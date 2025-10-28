use bios_basic::rbum::{
    dto::{
        rbum_cert_dto::RbumCertModifyReq,
        rbum_filer_dto::{RbumBasicFilterReq, RbumCertFilterReq, RbumRelFilterReq},
        rbum_rel_agg_dto::{RbumRelAggAddReq, RbumRelEnvAggAddReq},
        rbum_rel_dto::RbumRelAddReq,
    },
    rbum_enumeration::{RbumRelEnvKind, RbumRelFromKind},
    serv::{rbum_cert_serv::RbumCertServ, rbum_crud_serv::RbumCrudOperation, rbum_item_serv::RbumItemCrudOperation, rbum_rel_serv::RbumRelServ},
};
use std::collections::HashMap;
use itertools::Itertools;
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    chrono::DateTime,
    TardisFuns, TardisFunsInst,
};
use tardis::{
    chrono::{self, Utc},
    futures::future::join_all,
};

use crate::{
    basic::dto::{
        iam_cert_conf_dto::IamCertConfAkSkAddOrModifyReq,
        iam_cert_dto::IamCertAkSkAddReq,
        iam_filer_dto::IamResFilterReq,
        iam_open_dto::{IamOpenExtendData, IamOpenCertModifyReq, IamOpenCertStateKind, IamOpenAddOrModifyProductReq, IamOpenAkSkAddReq, IamOpenAkSkResp, IamOpenBindAkProductReq, IamOpenRuleResp},
        iam_res_dto::{IamResAddReq, IamResDetailResp, IamResModifyReq},
    },
    iam_config::IamConfig,
    iam_enumeration::{IamCertKernelKind, IamRelKind, IamResKind},
};

use super::{
    clients::iam_kv_client::IamKvClient, iam_cert_aksk_serv::IamCertAkSkServ, iam_cert_serv::IamCertServ, iam_key_cache_serv::IamIdentCacheServ, iam_rel_serv::IamRelServ,
    iam_res_serv::IamResServ, iam_tenant_serv::IamTenantServ,
};

pub struct IamOpenServ;

impl IamOpenServ {
    pub async fn add_or_modify_product(req: &IamOpenAddOrModifyProductReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(product) = Self::get_res_detail(req.code.as_ref(), IamResKind::Product, funs, ctx).await? {
            Self::modify_product(req, &product, funs, ctx).await?;
        } else {
            Self::add_product(req, funs, ctx).await?;
        }
        Ok(())
    }

    async fn modify_product(modify_req: &IamOpenAddOrModifyProductReq, product: &IamResDetailResp, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamResServ::modify_item(
            &product.id,
            &mut IamResModifyReq {
                name: Some(modify_req.name.clone()),
                icon: modify_req.icon.clone(),
                scope_level: modify_req.scope_level.clone(),
                disabled: modify_req.disabled,
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        for spec_req in &modify_req.specifications {
            if let Some(spec) = Self::get_res_detail(spec_req.code.as_ref(), IamResKind::Spec, funs, ctx).await? {
                IamResServ::modify_item(
                    &spec.id,
                    &mut IamResModifyReq {
                        name: Some(spec_req.name.clone()),
                        icon: spec_req.icon.clone(),
                        scope_level: spec_req.scope_level.clone(),
                        disabled: spec_req.disabled,
                        ext: spec_req.url.clone(),
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
                Self::set_rel_ak_cache(&spec.id, funs, ctx).await?;
                if let Some(bind_api_res) = spec_req.bind_api_res.clone() {
                    IamIdentCacheServ::add_or_modify_bind_api_res(&spec.id, bind_api_res, funs).await?;
                }
            } else {
                let spec_id = IamResServ::add_item(
                    &mut IamResAddReq {
                        code: spec_req.code.clone(),
                        name: spec_req.name.clone(),
                        kind: IamResKind::Spec,
                        scope_level: spec_req.scope_level.clone(),
                        disabled: spec_req.disabled,
                        ext: spec_req.url.clone(),
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?;
                IamRelServ::add_simple_rel(&IamRelKind::IamProductSpec, &product.id, &spec_id, None, None, false, false, funs, ctx).await?;
                Self::set_rel_ak_cache(&spec_id, funs, ctx).await?;
                if let Some(bind_api_res) = spec_req.bind_api_res.clone() {
                    IamIdentCacheServ::add_or_modify_bind_api_res(&spec_id, bind_api_res, funs).await?;
                }
            }
        }
        Ok(())
    }

    async fn get_res_detail(code: &str, kind: IamResKind, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<IamResDetailResp>> {
        IamResServ::find_one_detail_item(
            &IamResFilterReq {
                basic: RbumBasicFilterReq {
                    code: Some(format!("{}/*/{}", kind.to_int(), code)),
                    ..Default::default()
                },
                kind: Some(kind),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await
    }

    async fn add_product(add_req: &IamOpenAddOrModifyProductReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let product_id = IamResServ::add_item(
            &mut IamResAddReq {
                code: add_req.code.clone(),
                name: add_req.name.clone(),
                kind: IamResKind::Product,
                scope_level: add_req.scope_level.clone(),
                disabled: add_req.disabled,
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        for spec in &add_req.specifications {
            let spec_id = IamResServ::add_item(
                &mut IamResAddReq {
                    code: spec.code.clone(),
                    name: spec.name.clone(),
                    kind: IamResKind::Spec,
                    scope_level: spec.scope_level.clone(),
                    disabled: spec.disabled,
                    ext: spec.url.clone(),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            IamRelServ::add_simple_rel(&IamRelKind::IamProductSpec, &product_id, &spec_id, None, None, false, false, funs, ctx).await?;
            Self::set_rel_ak_cache(&spec_id, funs, ctx).await?;
            if let Some(bind_api_res) = spec.bind_api_res.clone() {
                IamIdentCacheServ::add_or_modify_bind_api_res(&spec_id, bind_api_res, funs).await?;
            }
        }
        Ok(())
    }

    pub async fn modify_cert(cert_id: &str, req: &IamOpenCertModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(state) = &req.state {
            let ak = RbumCertServ::find_one_detail_rbum(
                &RbumCertFilterReq {
                    id: Some(cert_id.to_string()),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            .ok_or_else(|| funs.err().internal_error("iam_open", "set_rules_cache", "illegal response", "401-iam-cert-code-not-exist"))?
            .ak;
            IamIdentCacheServ::add_or_modify_gateway_rule_info(&ak, &funs.conf::<IamConfig>().openapi_plugin_state, None, &state.to_string(), funs).await?;
        }
        Ok(())
    }

    pub async fn bind_cert_product_and_spec(cert_id: &str, bind_req: &IamOpenBindAkProductReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let old_product_rels = RbumRelServ::find_detail_rbums(
            &RbumRelFilterReq {
                from_rbum_id: Some(cert_id.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Cert),
                tag: Some(IamRelKind::IamCertProduct.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        for rel in old_product_rels {
            RbumRelServ::delete_rel_with_ext(&rel.id, funs, ctx).await?;
        }
        let old_spec_rels = RbumRelServ::find_detail_rbums(
            &RbumRelFilterReq {
                from_rbum_id: Some(cert_id.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Cert),
                tag: Some(IamRelKind::IamCertSpec.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        for rel in old_spec_rels {
            RbumRelServ::delete_rel_with_ext(&rel.id, funs, ctx).await?;
        }

        let product_id = IamResServ::find_one_detail_item(
            &IamResFilterReq {
                basic: RbumBasicFilterReq {
                    code: Some(format!("{}/*/{}", IamResKind::Product.to_int(), &bind_req.product_code)),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .ok_or_else(|| funs.err().internal_error("iam_open", "bind_cert_product_and_spec", "illegal response", "404-iam-res-not-exist"))?
        .id;
        let spec_id = IamResServ::find_one_detail_item(
            &IamResFilterReq {
                basic: RbumBasicFilterReq {
                    code: Some(format!("{}/*/{}", IamResKind::Spec.to_int(), &bind_req.spec_code)),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .ok_or_else(|| funs.err().internal_error("iam_open", "bind_cert_product_and_spec", "illegal response", "404-iam-res-not-exist"))?
        .id;

        let create_proj_id = if let Some(create_proj_code) = &bind_req.create_proj_code {
            Some(
                IamResServ::find_one_detail_item(
                    &IamResFilterReq {
                        basic: RbumBasicFilterReq {
                            code: Some(format!("{}/*/{}", IamResKind::Product.to_int(), create_proj_code)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?
                .ok_or_else(|| funs.err().internal_error("iam_open", "bind_cert_product_and_spec", "illegal response", "404-iam-res-not-exist"))?
                .id
            )
        } else {
            None
        };

        if let Some(create_proj_id) = &create_proj_id {
            let ak = RbumCertServ::find_one_detail_rbum(
                &RbumCertFilterReq {
                    id: Some(cert_id.to_string()),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            .ok_or_else(|| funs.err().internal_error("iam_open", "set_rules_cache", "illegal response", "401-iam-cert-code-not-exist"))?
            .ak;
            IamIdentCacheServ::set_open_api_extand_header(&ak, None, HashMap::from([("External-Id".to_string(), create_proj_id.clone())]), funs).await?;
        }
        Self::bind_cert_product(cert_id, &product_id, None, create_proj_id, funs, ctx).await?;
        Self::bind_cert_spec(
            cert_id,
            &spec_id,
            None,
            bind_req.start_time,
            bind_req.end_time,
            bind_req.api_call_frequency,
            bind_req.api_call_count,
            funs,
            ctx,
        )
        .await?;

        // update cert expire_sec
        if bind_req.end_time.is_some() && bind_req.start_time.is_some() {
            RbumCertServ::modify_rbum(
                cert_id,
                &mut RbumCertModifyReq {
                    start_time: bind_req.start_time,
                    end_time: bind_req.end_time,
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
        }

        Self::set_rules_cache(
            cert_id,
            &spec_id,
            bind_req.start_time,
            bind_req.end_time,
            bind_req.api_call_frequency,
            bind_req.api_call_count,
            funs,
            ctx,
        )
        .await?;
        Ok(())
    }

    async fn bind_cert_product(cert_id: &str, product_id: &str, own_paths: Option<String>, ext: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let req = &mut RbumRelAggAddReq {
            rel: RbumRelAddReq {
                tag: IamRelKind::IamCertProduct.to_string(),
                note: None,
                from_rbum_kind: RbumRelFromKind::Cert,
                from_rbum_id: cert_id.to_string(),
                to_rbum_item_id: product_id.to_string(),
                to_own_paths: own_paths.unwrap_or_else(|| ctx.own_paths.clone()),
                to_is_outside: true,
                ext,
                disabled: None,
            },
            attrs: vec![],
            envs: vec![],
        };
        RbumRelServ::add_rel(req, funs, ctx).await?;
        Ok(())
    }

    async fn bind_cert_spec(
        cert_id: &str,
        spec_id: &str,
        own_paths: Option<String>,
        start_time: Option<chrono::DateTime<Utc>>,
        end_time: Option<chrono::DateTime<Utc>>,
        api_call_frequency: Option<u32>,
        api_call_count: Option<u32>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let mut envs = vec![];
        if start_time.is_some() || end_time.is_some() {
            envs.push(RbumRelEnvAggAddReq {
                kind: RbumRelEnvKind::DatetimeRange,
                value1: start_time.unwrap().to_rfc3339(),
                value2: Some(end_time.unwrap().to_rfc3339()),
            });
        }
        if let Some(frequency) = api_call_frequency {
            envs.push(RbumRelEnvAggAddReq {
                kind: RbumRelEnvKind::CallFrequency,
                value1: frequency.to_string(),
                value2: None,
            });
        }
        if let Some(count) = api_call_count {
            envs.push(RbumRelEnvAggAddReq {
                kind: RbumRelEnvKind::CallCount,
                value1: count.to_string(),
                value2: None,
            });
        }
        let req = &mut RbumRelAggAddReq {
            rel: RbumRelAddReq {
                tag: IamRelKind::IamCertSpec.to_string(),
                note: None,
                from_rbum_kind: RbumRelFromKind::Cert,
                from_rbum_id: cert_id.to_string(),
                to_rbum_item_id: spec_id.to_string(),
                to_own_paths: own_paths.unwrap_or_else(|| ctx.own_paths.clone()),
                to_is_outside: true,
                ext: None,
                disabled: None,
            },
            attrs: vec![],
            envs,
        };
        RbumRelServ::add_rel(req, funs, ctx).await?;

        let ak = RbumCertServ::find_one_detail_rbum(
            &RbumCertFilterReq {
                id: Some(cert_id.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .ok_or_else(|| funs.err().internal_error("iam_open", "bind_cert_spec", "illegal response", "401-iam-cert-code-not-exist"))?
        .ak;

        IamIdentCacheServ::set_open_api_extand_data(&ak, None, IamOpenExtendData { id: spec_id.to_string() }, funs).await?;
        Ok(())
    }

    async fn set_rel_ak_cache(spec_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        for rel in RbumRelServ::find_detail_rbums(
            &RbumRelFilterReq {
                to_rbum_item_id: Some(spec_id.to_string()),
                tag: Some(IamRelKind::IamCertSpec.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?
        {
            Self::set_rules_cache(&rel.from_rbum_id, spec_id, None, None, None, None, funs, ctx).await?;
        }
        Ok(())
    }

    async fn set_rules_cache(
        cert_id: &str,
        spec_id: &str,
        start_time: Option<chrono::DateTime<Utc>>,
        end_time: Option<chrono::DateTime<Utc>>,
        api_call_frequency: Option<u32>,
        api_call_count: Option<u32>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let ak = RbumCertServ::find_one_detail_rbum(
            &RbumCertFilterReq {
                id: Some(cert_id.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .ok_or_else(|| funs.err().internal_error("iam_open", "set_rules_cache", "illegal response", "401-iam-cert-code-not-exist"))?
        .ak;
        if start_time.is_some() && end_time.is_some() {
            IamIdentCacheServ::add_or_modify_gateway_rule_info(
                &ak,
                &funs.conf::<IamConfig>().openapi_plugin_time_range,
                None,
                &format!("{},{}", start_time.unwrap().to_rfc3339(), end_time.unwrap().to_rfc3339()),
                funs,
            )
            .await?;
        }
        if let Some(count) = api_call_count {
            IamIdentCacheServ::add_or_modify_gateway_rule_info(&ak, &funs.conf::<IamConfig>().openapi_plugin_count, None, &count.to_string(), funs).await?;
        }
        let spec = IamResServ::find_one_detail_item(
            &IamResFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(vec![spec_id.to_string()]),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .ok_or_else(|| funs.err().internal_error("iam_open", "set_rules_cache", "illegal response", "404-iam-res-not-exist"))?;
        IamIdentCacheServ::add_or_modify_gateway_rule_info(&ak, &funs.conf::<IamConfig>().openapi_plugin_dynamic_route, None, &spec.ext, funs).await?;
        Ok(())
    }

    // 获取凭证规则信息(支持传入cert_id或ak)
    pub async fn get_rule_info(cert_id_req: Option<String>, ak_req: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamOpenRuleResp> {
        if cert_id_req.is_none() && ak_req.is_none() {
            return Err(funs.err().internal_error("iam_open", "get_rule_info", "illegal response", "404-iam-res-not-exist"));
        }
        let cert_id = if let Some(cert_id) = cert_id_req.clone() {
            cert_id
        } else {
            RbumCertServ::find_one_detail_rbum(
                &RbumCertFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ak: ak_req.clone(),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            .ok_or_else(|| funs.err().internal_error("iam_open", "get_rule_info", "cert not found", "404-iam-res-not-exist"))?
            .id
        };
        let ak = if let Some(ak) = ak_req.clone() {
            ak
        } else {
            RbumCertServ::find_one_detail_rbum(
                &RbumCertFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    id: cert_id_req.clone(),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
            .ok_or_else(|| funs.err().internal_error("iam_open", "get_rule_info", "cert not found", "404-iam-res-not-exist"))?
            .ak
        };
        // let spec_id = IamRelServ::find_from_id_rels(&IamRelKind::IamCertSpec, false, &cert_id, None, None, funs, ctx).await?.pop().unwrap_or_default();
        let spec_id = RbumRelServ::find_detail_rbums(
            &RbumRelFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                from_rbum_id: Some(cert_id.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Cert),
                tag: Some(IamRelKind::IamCertSpec.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?
        .pop()
        .map(|rel| rel.to_rbum_item_id)
        .unwrap_or_default();

        let spec_code = IamResServ::find_one_detail_item(
            &IamResFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ids: Some(vec![spec_id]),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .ok_or_else(|| funs.err().internal_error("iam_open", "set_rules_cache", "illegal response", "404-iam-res-not-exist"))?
        .code;
        let time_range = IamIdentCacheServ::get_gateway_rule_info(&ak, &funs.conf::<IamConfig>().openapi_plugin_time_range, None, funs).await?.unwrap_or_default();
        Ok(IamOpenRuleResp {
            cert_id,
            spec_code,
            start_time: if !time_range.is_empty() {
                time_range.split(',').collect_vec().first().map(|s| DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc))
            } else {
                None
            },
            end_time: if !time_range.is_empty() {
                time_range.split(',').collect_vec().last().map(|s| DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc))
            } else {
                None
            },
            api_call_frequency: IamIdentCacheServ::get_gateway_rule_info(&ak, &funs.conf::<IamConfig>().openapi_plugin_limit, None, funs).await?.map(|s| s.parse::<u32>().unwrap_or_default()),
            api_call_count: IamIdentCacheServ::get_gateway_rule_info(&ak, &funs.conf::<IamConfig>().openapi_plugin_count, None, funs).await?.map(|s| s.parse::<u32>().unwrap_or_default()),
            api_call_cumulative_count: IamIdentCacheServ::get_gateway_cumulative_count(&ak, None, funs).await?.map(|s| s.parse::<u32>().unwrap_or_default()),
        })
    }

    pub async fn general_cert(add_req: IamOpenAkSkAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamOpenAkSkResp> {
        let rel_iam_item_id = IamTenantServ::get_id_by_ctx(ctx, funs)?;
        let cert_conf = IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::AkSk.to_string().as_str(), Some(rel_iam_item_id.clone()), funs).await;
        let cert_conf_id = if let Ok(cert_conf_id) = cert_conf {
            cert_conf_id
        } else {
            IamCertAkSkServ::add_cert_conf(
                &IamCertConfAkSkAddOrModifyReq {
                    name: TrimString(format!("AkSk-{}", &rel_iam_item_id)),
                    expire_sec: Some(0),
                },
                Some(IamTenantServ::get_id_by_ctx(ctx, funs)?),
                funs,
                ctx,
            )
            .await?
        };
        let ak = TardisFuns::crypto.key.generate_ak()?;
        let sk = TardisFuns::crypto.key.generate_sk(&ak)?;

        let cert_id = IamCertAkSkServ::add_cert(
            &IamCertAkSkAddReq {
                tenant_id: add_req.tenant_id,
                app_id: add_req.app_id,
            },
            &ak,
            &sk,
            &cert_conf_id,
            funs,
            ctx,
        )
        .await?;

        IamIdentCacheServ::add_or_modify_gateway_rule_info(&ak, &funs.conf::<IamConfig>().openapi_plugin_state, None, &add_req.state.unwrap_or(IamOpenCertStateKind::Enabled).to_string(), funs).await?;
        Ok(IamOpenAkSkResp { id: cert_id, ak, sk })
    }

    pub async fn refresh_cert_cumulative_count(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let cert_ids = RbumRelServ::find_rels(
            &RbumRelFilterReq {
                tag: Some(IamRelKind::IamCertSpec.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?
        .into_iter()
        .map(|rel_agg| rel_agg.rel.from_rbum_id)
        .collect_vec();
        let _ = join_all(
            cert_ids
                .into_iter()
                .map(|cert_id| async move {
                    if let Ok(Some(cert)) = RbumCertServ::find_one_detail_rbum(
                        &RbumCertFilterReq {
                            id: Some(cert_id.to_string()),
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await
                    {
                        let count = IamIdentCacheServ::get_gateway_cumulative_count(&cert.ak, None, funs).await.unwrap_or_default();
                        let _ = IamKvClient::add_or_modify_key_name(
                            &funs.conf::<IamConfig>().spi.kv_api_call_count_prefix.clone(),
                            cert.id.as_str(),
                            &count.unwrap_or_default(),
                            None,
                            funs,
                            ctx,
                        )
                        .await;
                    }
                })
                .collect_vec(),
        )
        .await;
        Ok(())
    }

    pub async fn refresh_open_cache(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let specs = IamResServ::find_detail_items(
            &IamResFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                kind: Some(IamResKind::Spec),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        for spec in specs {
            let rel_cert_ids = RbumRelServ::find_detail_rbums(
                &RbumRelFilterReq {
                    to_rbum_item_id: Some(spec.id.to_string()),
                    tag: Some(IamRelKind::IamCertSpec.to_string()),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            ).await?.into_iter().map(|rel| rel.from_rbum_id).collect_vec();
            for cert_id in rel_cert_ids {
                if let Some(cert) = RbumCertServ::find_one_detail_rbum(
                    &RbumCertFilterReq {
                        id: Some(cert_id.to_string()),
                        basic: RbumBasicFilterReq {
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    funs,
                    ctx,
                ).await? {
                    // 1、api调用频率缓存重写
                    if let Some(api_call_frequency) = IamIdentCacheServ::get_gateway_rule_info(&cert.ak, &funs.conf::<IamConfig>().openapi_plugin_limit, None, funs).await?.map(|s| s.parse::<u32>().unwrap_or_default()) {
                        IamIdentCacheServ::set_open_api_call_frequency(&spec.id, None, api_call_frequency.as_str(), funs).await?;
                    };
                    // 2、新增ak关联规格ID
                    IamIdentCacheServ::set_open_api_extand_data(&cert.ak, None, IamOpenExtendData { id: spec.id.clone() }, funs).await?;
                    // 3、新增ak可用状态
                    IamIdentCacheServ::add_or_modify_gateway_rule_info(&cert.ak, &funs.conf::<IamConfig>().openapi_plugin_state, None, IamOpenCertStateKind::Enabled.to_string().as_str(), funs).await?;
                }
            }
        }

        Ok(())
    }
}
