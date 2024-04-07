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
    log::{error, info},
    tokio,
    web::{web_server::TardisWebServer, ws_client::TardisWSClient},
    TardisFuns, TardisFunsInst,
};

use crate::{
    api::{
        cc::{flow_cc_inst_api, flow_cc_model_api, flow_cc_state_api},
        ci::{flow_ci_inst_api, flow_ci_model_api, flow_ci_state_api},
        cs::flow_cs_config_api,
    },
    domain::{flow_inst, flow_model, flow_state, flow_transition},
    dto::{
        flow_model_dto::FlowModelFilterReq,
        flow_state_dto::FlowSysStateKind,
        flow_transition_dto::{FlowTransitionDoubleCheckInfo, FlowTransitionInitInfo},
    },
    flow_config::{BasicInfo, FlowBasicInfoManager, FlowConfig},
    flow_constants,
    serv::flow_model_serv::FlowModelServ,
};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    let funs = flow_constants::get_tardis_inst();
    init_db(funs).await?;
    tokio::spawn(init_event());
    init_api(web_server).await
}

async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server
        .add_module(
            flow_constants::DOMAIN_CODE,
            (
                flow_cc_state_api::FlowCcStateApi,
                flow_cc_model_api::FlowCcModelApi,
                flow_cc_inst_api::FlowCcInstApi,
                flow_cs_config_api::FlowCsConfigApi,
                flow_ci_inst_api::FlowCiInstApi,
                flow_ci_model_api::FlowCiModelApi,
                flow_ci_state_api::FlowCiStateApi,
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

async fn init_basic_info<'a>(funs: &TardisFunsInst) -> TardisResult<()> {
    let kind_state_id = RbumKindServ::get_rbum_kind_id_by_code(flow_constants::RBUM_KIND_STATE_CODE, funs)
        .await?
        .ok_or_else(|| funs.err().not_found("flow", "init", "not found state kind", ""))?;
    let kind_model_id = RbumKindServ::get_rbum_kind_id_by_code(flow_constants::RBUM_KIND_MODEL_CODE, funs)
        .await?
        .ok_or_else(|| funs.err().not_found("flow", "init", "not found model kind", ""))?;

    let domain_flow_id =
        RbumDomainServ::get_rbum_domain_id_by_code(flow_constants::DOMAIN_CODE, funs).await?.ok_or_else(|| funs.err().not_found("flow", "init", "not found flow domain", ""))?;

    FlowBasicInfoManager::set(BasicInfo {
        kind_state_id,
        kind_model_id,
        domain_flow_id,
    })?;
    Ok(())
}

pub async fn init_rbum_data(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let kind_state_id = add_kind(flow_constants::RBUM_KIND_STATE_CODE, flow_constants::RBUM_EXT_TABLE_STATE, funs, ctx).await?;
    let kind_model_id = add_kind(flow_constants::RBUM_KIND_MODEL_CODE, flow_constants::RBUM_EXT_TABLE_MODEL, funs, ctx).await?;

    let domain_flow_id = add_domain(funs, ctx).await?;

    FlowBasicInfoManager::set(BasicInfo {
        kind_state_id,
        kind_model_id,
        domain_flow_id,
    })?;

    info!("Flow initialization is complete.",);
    Ok(())
}

async fn add_kind<'a>(scheme: &str, ext_table: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
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

async fn add_domain<'a>(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
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

pub async fn truncate_data<'a>(funs: &TardisFunsInst) -> TardisResult<()> {
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
        FlowModelServ::init_model(
            "TICKET",
            vec![
                ("待处理", FlowSysStateKind::Start, ""),
                ("处理中", FlowSysStateKind::Progress, ""),
                ("待确认", FlowSysStateKind::Progress, ""),
                ("已关闭", FlowSysStateKind::Finish, ""),
                ("已撤销", FlowSysStateKind::Finish, ""),
            ],
            "待处理-处理中-待确认-已关闭-已撤销",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_name: "待处理".to_string(),
                    to_flow_state_name: "处理中".to_string(),
                    name: "立即处理".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成处理中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待处理".to_string(),
                    to_flow_state_name: "已撤销".to_string(),
                    name: "撤销".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已撤销？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待处理".to_string(),
                    to_flow_state_name: "待确认".to_string(),
                    name: "处理完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成待确认？".to_string()),
                    }),
                    guard_by_his_operators: Some(true),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待处理".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待确认".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "确认解决".into(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待确认".to_string(),
                    to_flow_state_name: "处理中".to_string(),
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
        // 需求模板初始化
        FlowModelServ::init_model(
            "REQ",
            vec![
                ("待开始", FlowSysStateKind::Start, ""),
                ("进行中", FlowSysStateKind::Progress, ""),
                ("已完成", FlowSysStateKind::Finish, ""),
                ("已关闭", FlowSysStateKind::Finish, ""),
            ],
            "待开始-进行中-已完成-已关闭",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "开始".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已完成？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "重新处理".into(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已关闭".to_string(),
                    to_flow_state_name: "待开始".to_string(),
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
        FlowModelServ::init_model(
            "PROJ",
            vec![
                ("待开始", FlowSysStateKind::Start, ""),
                ("进行中", FlowSysStateKind::Progress, ""),
                ("存在风险", FlowSysStateKind::Progress, ""),
                ("已完成", FlowSysStateKind::Progress, ""),
                ("已关闭", FlowSysStateKind::Finish, ""),
                ("已归档", FlowSysStateKind::Finish, ""),
            ],
            "待开始-进行中-存在风险-已完成-已关闭-已归档",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "开始".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已完成？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "存在风险".to_string(),
                    name: "有风险".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成存在风险？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "正常".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成处理中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "重新处理".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "已归档".to_string(),
                    name: "归档".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已归档？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已关闭".to_string(),
                    to_flow_state_name: "待开始".to_string(),
                    name: "激活".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成待开始？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已关闭".to_string(),
                    to_flow_state_name: "已归档".to_string(),
                    name: "归档".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已归档？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已归档".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "重新激活".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已归档".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
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
        FlowModelServ::init_model(
            "ITER",
            vec![
                ("待开始", FlowSysStateKind::Start, ""),
                ("进行中", FlowSysStateKind::Progress, ""),
                ("存在风险", FlowSysStateKind::Progress, ""),
                ("已完成", FlowSysStateKind::Progress, ""),
                ("已关闭", FlowSysStateKind::Finish, ""),
            ],
            "待开始-进行中-存在风险-已完成-已关闭",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "开始".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已完成？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "存在风险".to_string(),
                    name: "有风险".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成存在风险？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "正常".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已完成？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "重新处理".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已关闭".to_string(),
                    to_flow_state_name: "待开始".to_string(),
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
        FlowModelServ::init_model(
            "TASK",
            vec![
                ("待开始", FlowSysStateKind::Start, ""),
                ("进行中", FlowSysStateKind::Progress, ""),
                ("存在风险", FlowSysStateKind::Progress, ""),
                ("已完成", FlowSysStateKind::Progress, ""),
                ("已关闭", FlowSysStateKind::Finish, ""),
            ],
            "待开始-进行中-存在风险-已完成-已关闭",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "开始".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已完成？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "存在风险".to_string(),
                    name: "有风险".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认该任务存在风险？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "正常".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已完成？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "重新处理".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成进行中？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将状态修改成已关闭？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已关闭".to_string(),
                    to_flow_state_name: "待开始".to_string(),
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

    Ok(())
}

async fn init_event() -> TardisResult<()> {
    let funs = flow_constants::get_tardis_inst();
    let conf = funs.conf::<FlowConfig>();
    if let Some(event_config) = conf.event.as_ref() {
        loop {
            if TardisFuns::web_server().is_running().await {
                break;
            } else {
                tokio::task::yield_now().await
            }
        }
        crate::event::start_flow_event_service(event_config).await?;
    }
    Ok(())
}

async fn init_ws_flow_client() -> Option<TardisWSClient> {
    while !TardisFuns::web_server().is_running().await {
        tardis::tokio::task::yield_now().await
    }
    let funs = flow_constants::get_tardis_inst();
    let conf = funs.conf::<FlowConfig>();
    if conf.event.is_none() {
        set_default_flow_avatar("".to_owned());
        return None;
    }
    let mut event_conf = conf.event.clone().unwrap();
    if !event_conf.in_event {
        set_default_flow_avatar("".to_owned());
        return None;
    }
    if event_conf.avatars.is_empty() {
        event_conf.avatars.push(format!("{}/{}", event_conf.topic_code, tardis::pkg!()))
    }
    let default_avatar = event_conf.avatars[0].clone();
    set_default_flow_avatar(default_avatar);
    let client = bios_sdk_invoke::clients::event_client::EventClient::new(&event_conf.base_url, &funs);
    loop {
        let addr = loop {
            if let Ok(result) = client.register(&event_conf.clone().into()).await {
                break result.ws_addr;
            }
            tardis::tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        };
        let ws_client = TardisFuns::ws_client(&addr, |_| async move { None }).await;
        match ws_client {
            Ok(ws_client) => {
                return Some(ws_client);
            }
            Err(err) => {
                error!("[BIOS.Iam] failed to connect to event server: {}", err);
                tardis::tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        }
    }
}

use std::sync::OnceLock;
tardis::tardis_static! {
    pub(crate) async ws_flow_client: Option<TardisWSClient> = init_ws_flow_client();
    pub(crate) async set default_flow_avatar: String;
}
