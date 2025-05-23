use bios_basic::rbum::{
    dto::{rbum_domain_dto::RbumDomainAddReq, rbum_filer_dto::RbumBasicFilterReq, rbum_kind_dto::RbumKindAddReq},
    rbum_enumeration::RbumScopeLevelKind,
    rbum_initializer,
    serv::{rbum_crud_serv::RbumCrudOperation, rbum_domain_serv::RbumDomainServ, rbum_item_serv::RbumItemCrudOperation, rbum_kind_serv::RbumKindServ},
};
use bios_sdk_invoke::invoke_initializer;

use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    db::{reldb_client::TardisActiveModel, sea_orm::sea_query::Table},
    log::info,
    web::web_server::TardisWebServer,
    TardisFuns, TardisFunsInst,
};

use crate::{
    api::{
        ca::flow_ca_model_api,
        cc::{flow_cc_inst_api, flow_cc_model_api, flow_cc_model_version_api, flow_cc_state_api},
        ci::{flow_ci_inst_api, flow_ci_model_api, flow_ci_state_api, flow_ci_sub_deploy_api},
        cs::flow_cs_config_api,
        ct::flow_ct_model_api,
    },
    domain::{flow_inst, flow_model, flow_model_version, flow_state, flow_transition},
    dto::{
        flow_model_dto::FlowModelFilterReq,
        flow_state_dto::FlowSysStateKind,
        flow_transition_dto::{
            FlowTransitionDoubleCheckInfo, FlowTransitionFrontActionInfo, FlowTransitionFrontActionInfoRelevanceRelation, FlowTransitionFrontActionRightValue,
            FlowTransitionInitInfo,
        },
    },
    flow_config::{BasicInfo, FlowBasicInfoManager, FlowConfig},
    flow_constants,
    serv::{flow_model_serv::FlowModelServ, flow_state_serv::FlowStateServ},
};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    let funs = flow_constants::get_tardis_inst();
    init_db(funs).await?;
    init_api(web_server).await
}

async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server
        .add_module(
            flow_constants::DOMAIN_CODE,
            (
                flow_ca_model_api::FlowCaModelApi,
                flow_ct_model_api::FlowCtModelApi,
                flow_cc_state_api::FlowCcStateApi,
                flow_cc_model_api::FlowCcModelApi,
                flow_cc_model_version_api::FlowCcModelVersionApi,
                flow_cc_inst_api::FlowCcInstApi,
                flow_cs_config_api::FlowCsConfigApi,
                flow_ci_inst_api::FlowCiInstApi,
                flow_ci_model_api::FlowCiModelApi,
                flow_ci_state_api::FlowCiStateApi,
                flow_ci_sub_deploy_api::FlowCiSubDeployApi,
            ),
        )
        .await;
    Ok(())
}

pub async fn init_db(mut funs: TardisFunsInst) -> TardisResult<()> {
    bios_basic::rbum::rbum_initializer::init(funs.module_code(), funs.conf::<FlowConfig>().rbum.clone()).await?;
    invoke_initializer::init(funs.module_code(), funs.conf::<FlowConfig>().invoke.clone())?;
    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };
    funs.begin().await?;
    if check_initialized(&funs, &ctx).await? {
        init_basic_info(&funs).await?;
    } else {
        let db_kind = TardisFuns::reldb().backend();
        let compatible_type = TardisFuns::reldb().compatible_type();
        funs.db().init(flow_state::ActiveModel::init(db_kind, None, compatible_type)).await?;
        funs.db().init(flow_model::ActiveModel::init(db_kind, None, compatible_type)).await?;
        funs.db().init(flow_model_version::ActiveModel::init(db_kind, None, compatible_type)).await?;
        funs.db().init(flow_transition::ActiveModel::init(db_kind, None, compatible_type)).await?;
        funs.db().init(flow_inst::ActiveModel::init(db_kind, None, compatible_type)).await?;
        init_rbum_data(&funs, &ctx).await?;
    };
    funs.commit().await?;
    Ok(())
}

async fn check_initialized(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
    RbumDomainServ::exist_rbum(
        &RbumBasicFilterReq {
            ignore_scope: true,
            rel_ctx_owner: false,
            code: Some(flow_constants::DOMAIN_CODE.to_string()),
            ..Default::default()
        },
        funs,
        ctx,
    )
    .await
}

async fn init_basic_info(funs: &TardisFunsInst) -> TardisResult<()> {
    let kind_state_id = RbumKindServ::get_rbum_kind_id_by_code(flow_constants::RBUM_KIND_STATE_CODE, funs)
        .await?
        .ok_or_else(|| funs.err().not_found("flow", "init", "not found state kind", ""))?;
    let kind_model_id = RbumKindServ::get_rbum_kind_id_by_code(flow_constants::RBUM_KIND_MODEL_CODE, funs)
        .await?
        .ok_or_else(|| funs.err().not_found("flow", "init", "not found model kind", ""))?;
    let kind_model_version_id = RbumKindServ::get_rbum_kind_id_by_code(flow_constants::RBUM_KIND_MODEL_VERSION_CODE, funs)
        .await?
        .ok_or_else(|| funs.err().not_found("flow", "init", "not found model kind", ""))?;

    let domain_flow_id =
        RbumDomainServ::get_rbum_domain_id_by_code(flow_constants::DOMAIN_CODE, funs).await?.ok_or_else(|| funs.err().not_found("flow", "init", "not found flow domain", ""))?;

    FlowBasicInfoManager::set(BasicInfo {
        kind_state_id,
        kind_model_id,
        kind_model_version_id,
        domain_flow_id,
    })?;
    Ok(())
}

pub async fn init_rbum_data(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let kind_state_id = add_kind(flow_constants::RBUM_KIND_STATE_CODE, flow_constants::RBUM_EXT_TABLE_STATE, funs, ctx).await?;
    let kind_model_id = add_kind(flow_constants::RBUM_KIND_MODEL_CODE, flow_constants::RBUM_EXT_TABLE_MODEL, funs, ctx).await?;
    let kind_model_version_id = add_kind(flow_constants::RBUM_KIND_MODEL_VERSION_CODE, flow_constants::RBUM_EXT_TABLE_MODEL_VERSION, funs, ctx).await?;

    let domain_flow_id = add_domain(funs, ctx).await?;

    FlowBasicInfoManager::set(BasicInfo {
        kind_state_id,
        kind_model_id,
        kind_model_version_id,
        domain_flow_id,
    })?;

    info!("Flow initialization is complete.",);
    Ok(())
}

async fn add_kind(scheme: &str, ext_table: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
    RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString(scheme.to_string()),
            name: TrimString(scheme.to_string()),
            note: None,
            icon: None,
            sort: None,
            module: None,
            ext_table_name: Some(ext_table.to_string().to_lowercase()),
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        funs,
        ctx,
    )
    .await
}

async fn add_domain(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
    RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString(flow_constants::DOMAIN_CODE.to_string()),
            name: TrimString(flow_constants::DOMAIN_CODE.to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        funs,
        ctx,
    )
    .await
}

pub async fn truncate_data(funs: &TardisFunsInst) -> TardisResult<()> {
    rbum_initializer::truncate_data(funs).await?;
    funs.db().execute(Table::truncate().table(flow_state::Entity)).await?;
    funs.db().execute(Table::truncate().table(flow_model::Entity)).await?;
    funs.db().execute(Table::truncate().table(flow_transition::Entity)).await?;
    funs.db().execute(Table::truncate().table(flow_inst::Entity)).await?;
    funs.cache().flushdb().await?;
    Ok(())
}

pub async fn init_flow_model(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let ticket_init_model = FlowModelServ::paginate_items(
        &FlowModelFilterReq {
            basic: RbumBasicFilterReq { ..Default::default() },
            tags: Some(vec!["TICKET".to_string()]),
            ..Default::default()
        },
        1,
        1,
        None,
        None,
        funs,
        ctx,
    )
    .await?
    .records
    .pop();
    if ticket_init_model.is_none() {
        // 工单模板初始化
        let mut bind_states = vec![];
        bind_states.push(FlowStateServ::init_state("TICKET", "待处理", FlowSysStateKind::Start, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("TICKET", "处理中", FlowSysStateKind::Progress, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("TICKET", "待确认", FlowSysStateKind::Progress, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("TICKET", "已关闭", FlowSysStateKind::Finish, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("TICKET", "已撤销", FlowSysStateKind::Finish, "", funs, ctx).await?);
        FlowModelServ::init_model(
            "TICKET",
            bind_states[0].clone(),
            bind_states.clone(),
            "待处理-处理中-待确认-已关闭-已撤销",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[0].clone(),
                    to_flow_state_id: bind_states[1].clone(),
                    name: "立即处理".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成处理中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[0].clone(),
                    to_flow_state_id: bind_states[4].clone(),
                    name: "撤销".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已撤销？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[0].clone(),
                    to_flow_state_id: bind_states[2].clone(),
                    name: "处理完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成待确认？".to_string()),
                    }),
                    guard_by_his_operators: Some(true),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[0].clone(),
                    to_flow_state_id: bind_states[3].clone(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[2].clone(),
                    to_flow_state_id: bind_states[3].clone(),
                    name: "确认解决".into(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[2].clone(),
                    to_flow_state_id: bind_states[1].clone(),
                    name: "未解决".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成处理中？".to_string()),
                    }),
                    ..Default::default()
                },
            ],
            funs,
            ctx,
        )
        .await?;
    }
    let req_init_model = FlowModelServ::paginate_items(
        &FlowModelFilterReq {
            basic: RbumBasicFilterReq { ..Default::default() },
            tags: Some(vec!["REQ".to_string()]),
            ..Default::default()
        },
        1,
        1,
        None,
        None,
        funs,
        ctx,
    )
    .await?
    .records
    .pop();
    if req_init_model.is_none() {
        let mut bind_states = vec![];
        bind_states.push(FlowStateServ::init_state("REQ", "待开始", FlowSysStateKind::Start, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("REQ", "进行中", FlowSysStateKind::Progress, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("REQ", "已完成", FlowSysStateKind::Finish, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("REQ", "已关闭", FlowSysStateKind::Finish, "", funs, ctx).await?);
        // 需求模板初始化
        FlowModelServ::init_model(
            "REQ",
            bind_states[0].clone(),
            bind_states.clone(),
            "待开始-进行中-已完成-已关闭",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[0].clone(),
                    to_flow_state_id: bind_states[1].clone(),
                    name: "开始".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[0].clone(),
                    to_flow_state_id: bind_states[3].clone(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[1].clone(),
                    to_flow_state_id: bind_states[2].clone(),
                    name: "完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已完成？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[1].clone(),
                    to_flow_state_id: bind_states[3].clone(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[2].clone(),
                    to_flow_state_id: bind_states[1].clone(),
                    name: "重新处理".into(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[2].clone(),
                    to_flow_state_id: bind_states[3].clone(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[3].clone(),
                    to_flow_state_id: bind_states[0].clone(),
                    name: "激活".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成待开始？".to_string()),
                    }),
                    ..Default::default()
                },
            ],
            funs,
            ctx,
        )
        .await?;
    }
    let product_init_model = FlowModelServ::paginate_items(
        &FlowModelFilterReq {
            basic: RbumBasicFilterReq { ..Default::default() },
            tags: Some(vec!["PROJ".to_string()]),
            ..Default::default()
        },
        1,
        1,
        None,
        None,
        funs,
        ctx,
    )
    .await?
    .records
    .pop();
    if product_init_model.is_none() {
        let mut bind_states = vec![];
        bind_states.push(FlowStateServ::init_state("PROJ", "待开始", FlowSysStateKind::Start, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("PROJ", "进行中", FlowSysStateKind::Progress, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("PROJ", "存在风险", FlowSysStateKind::Progress, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("PROJ", "已完成", FlowSysStateKind::Progress, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("PROJ", "已关闭", FlowSysStateKind::Finish, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("PROJ", "已归档", FlowSysStateKind::Finish, "", funs, ctx).await?);
        FlowModelServ::init_model(
            "PROJ",
            bind_states[0].clone(),
            bind_states.clone(),
            "待开始-进行中-存在风险-已完成-已关闭-已归档",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[0].clone(),
                    to_flow_state_id: bind_states[1].clone(),
                    name: "开始".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[0].clone(),
                    to_flow_state_id: bind_states[4].clone(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[1].clone(),
                    to_flow_state_id: bind_states[3].clone(),
                    name: "完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已完成？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[1].clone(),
                    to_flow_state_id: bind_states[2].clone(),
                    name: "有风险".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成存在风险？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[1].clone(),
                    to_flow_state_id: bind_states[4].clone(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[2].clone(),
                    to_flow_state_id: bind_states[1].clone(),
                    name: "正常".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[2].clone(),
                    to_flow_state_id: bind_states[3].clone(),
                    name: "完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成处理中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[2].clone(),
                    to_flow_state_id: bind_states[4].clone(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[3].clone(),
                    to_flow_state_id: bind_states[1].clone(),
                    name: "重新处理".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[3].clone(),
                    to_flow_state_id: bind_states[4].clone(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[3].clone(),
                    to_flow_state_id: bind_states[5].clone(),
                    name: "归档".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已归档？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[4].clone(),
                    to_flow_state_id: bind_states[0].clone(),
                    name: "激活".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成待开始？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[4].clone(),
                    to_flow_state_id: bind_states[5].clone(),
                    name: "归档".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已归档？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[5].clone(),
                    to_flow_state_id: bind_states[1].clone(),
                    name: "重新激活".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[5].clone(),
                    to_flow_state_id: bind_states[4].clone(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
            ],
            funs,
            ctx,
        )
        .await?;
    }
    let iter_init_model = FlowModelServ::paginate_items(
        &FlowModelFilterReq {
            basic: RbumBasicFilterReq { ..Default::default() },
            tags: Some(vec!["ITER".to_string()]),
            ..Default::default()
        },
        1,
        1,
        None,
        None,
        funs,
        ctx,
    )
    .await?
    .records
    .pop();
    if iter_init_model.is_none() {
        let mut bind_states = vec![];
        bind_states.push(FlowStateServ::init_state("ITER", "待开始", FlowSysStateKind::Start, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("ITER", "进行中", FlowSysStateKind::Progress, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("ITER", "存在风险", FlowSysStateKind::Progress, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("ITER", "已完成", FlowSysStateKind::Progress, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("ITER", "已关闭", FlowSysStateKind::Finish, "", funs, ctx).await?);
        FlowModelServ::init_model(
            "ITER",
            bind_states[0].clone(),
            bind_states.clone(),
            "待开始-进行中-存在风险-已完成-已关闭",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[0].clone(),
                    to_flow_state_id: bind_states[1].clone(),
                    name: "开始".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[0].clone(),
                    to_flow_state_id: bind_states[4].clone(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[1].clone(),
                    to_flow_state_id: bind_states[3].clone(),
                    name: "完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已完成？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[1].clone(),
                    to_flow_state_id: bind_states[2].clone(),
                    name: "有风险".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成存在风险？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[1].clone(),
                    to_flow_state_id: bind_states[4].clone(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[2].clone(),
                    to_flow_state_id: bind_states[1].clone(),
                    name: "正常".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[2].clone(),
                    to_flow_state_id: bind_states[3].clone(),
                    name: "完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已完成？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[2].clone(),
                    to_flow_state_id: bind_states[4].clone(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[3].clone(),
                    to_flow_state_id: bind_states[1].clone(),
                    name: "重新处理".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[3].clone(),
                    to_flow_state_id: bind_states[4].clone(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[4].clone(),
                    to_flow_state_id: bind_states[0].clone(),
                    name: "激活".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成待开始？".to_string()),
                    }),
                    ..Default::default()
                },
            ],
            funs,
            ctx,
        )
        .await?;
    }
    let task_init_model = FlowModelServ::paginate_items(
        &FlowModelFilterReq {
            basic: RbumBasicFilterReq { ..Default::default() },
            tags: Some(vec!["TASK".to_string()]),
            ..Default::default()
        },
        1,
        1,
        None,
        None,
        funs,
        ctx,
    )
    .await?
    .records
    .pop();
    if task_init_model.is_none() {
        let mut bind_states = vec![];
        bind_states.push(FlowStateServ::init_state("TASK", "待开始", FlowSysStateKind::Start, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("TASK", "进行中", FlowSysStateKind::Progress, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("TASK", "存在风险", FlowSysStateKind::Progress, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("TASK", "已完成", FlowSysStateKind::Progress, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("TASK", "已关闭", FlowSysStateKind::Finish, "", funs, ctx).await?);
        FlowModelServ::init_model(
            "TASK",
            bind_states[0].clone(),
            bind_states.clone(),
            "待开始-进行中-存在风险-已完成-已关闭",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[0].clone(),
                    to_flow_state_id: bind_states[1].clone(),
                    name: "开始".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[0].clone(),
                    to_flow_state_id: bind_states[4].clone(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[1].clone(),
                    to_flow_state_id: bind_states[3].clone(),
                    name: "完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已完成？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[1].clone(),
                    to_flow_state_id: bind_states[2].clone(),
                    name: "有风险".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认该任务存在风险？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[1].clone(),
                    to_flow_state_id: bind_states[4].clone(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[2].clone(),
                    to_flow_state_id: bind_states[1].clone(),
                    name: "正常".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[2].clone(),
                    to_flow_state_id: bind_states[3].clone(),
                    name: "完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已完成？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[2].clone(),
                    to_flow_state_id: bind_states[4].clone(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[3].clone(),
                    to_flow_state_id: bind_states[1].clone(),
                    name: "重新处理".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[3].clone(),
                    to_flow_state_id: bind_states[4].clone(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[4].clone(),
                    to_flow_state_id: bind_states[0].clone(),
                    name: "激活".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成待开始？".to_string()),
                    }),
                    ..Default::default()
                },
            ],
            funs,
            ctx,
        )
        .await?;
    }
    let review_init_model = FlowModelServ::paginate_items(
        &FlowModelFilterReq {
            basic: RbumBasicFilterReq { ..Default::default() },
            tags: Some(vec!["REVIEW".to_string()]),
            ..Default::default()
        },
        1,
        1,
        None,
        None,
        funs,
        ctx,
    )
    .await?
    .records
    .pop();
    if review_init_model.is_none() {
        let mut bind_states = vec![];
        bind_states.push(FlowStateServ::init_state("REVIEW", "待开始", FlowSysStateKind::Start, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("REVIEW", "进行中", FlowSysStateKind::Progress, "", funs, ctx).await?);
        bind_states.push(FlowStateServ::init_state("REVIEW", "已结束", FlowSysStateKind::Finish, "", funs, ctx).await?);
        FlowModelServ::init_model(
            "REVIEW",
            bind_states[0].clone(),
            bind_states.clone(),
            "评审通用审批流",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[0].clone(),
                    to_flow_state_id: bind_states[1].clone(),
                    name: "发起评审".to_string(),
                    action_by_front_changes: vec![FlowTransitionFrontActionInfo {
                        relevance_relation: FlowTransitionFrontActionInfoRelevanceRelation::Lt,
                        relevance_label: "".to_string(),
                        left_value: "review_start_time".to_string(),
                        left_label: "".to_string(),
                        right_value: FlowTransitionFrontActionRightValue::RealTime,
                        select_field: None,
                        select_field_label: None,
                        change_content: None,
                        change_content_label: None,
                    }],
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[0].clone(),
                    to_flow_state_id: bind_states[2].clone(),
                    name: "结束评审".to_string(),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_id: bind_states[1].clone(),
                    to_flow_state_id: bind_states[2].clone(),
                    name: "结束评审".to_string(),
                    double_check: None,
                    action_by_front_changes: vec![FlowTransitionFrontActionInfo {
                        relevance_relation: FlowTransitionFrontActionInfoRelevanceRelation::Gt,
                        relevance_label: "".to_string(),
                        left_value: "review_end_time".to_string(),
                        left_label: "".to_string(),
                        right_value: FlowTransitionFrontActionRightValue::RealTime,
                        select_field: None,
                        select_field_label: None,
                        change_content: None,
                        change_content_label: None,
                    }],
                    ..Default::default()
                },
            ],
            funs,
            ctx,
        )
        .await?;
    }

    Ok(())
}
