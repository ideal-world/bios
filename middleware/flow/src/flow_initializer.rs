use bios_basic::rbum::{
    dto::{rbum_domain_dto::RbumDomainAddReq, rbum_filer_dto::RbumBasicFilterReq, rbum_kind_dto::RbumKindAddReq},
    rbum_enumeration::RbumScopeLevelKind,
    rbum_initializer,
    serv::{
        rbum_crud_serv::RbumCrudOperation,
        rbum_domain_serv::RbumDomainServ,
        rbum_item_serv::{RbumItemCrudOperation, RbumItemServ},
        rbum_kind_serv::RbumKindServ,
    },
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
        cc::{flow_cc_inst_api, flow_cc_model_api, flow_cc_state_api},
        ci::flow_ci_inst_api,
        cs::flow_cs_config_api,
    },
    domain::{flow_config, flow_inst, flow_model, flow_state, flow_transition},
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
        init_model(&funs, &ctx).await?;
    } else {
        let db_kind = TardisFuns::reldb().backend();
        let compatible_type = TardisFuns::reldb().compatible_type();
        funs.db().init(flow_state::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
        funs.db().init(flow_model::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
        funs.db().init(flow_transition::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
        funs.db().init(flow_inst::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
        funs.db().init(flow_config::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
        init_rbum_data(&funs, &ctx).await?;
        init_model(&funs, &ctx).await?;
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
    funs.cache().flushdb().await?;
    Ok(())
}
async fn get_role_id(role_code: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
    Ok(RbumItemServ::find_id_rbums(
        &RbumBasicFilterReq {
            code: Some(role_code.to_string()),
            ..Default::default()
        },
        None,
        None,
        funs,
        ctx,
    )
    .await?
    .pop()
    .unwrap_or_default())
}

async fn init_model(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let admin_role_id = get_role_id("app_admin", funs, ctx).await?; // 项目负责人
    let iter_admin_role_id = get_role_id("app_admin_iterate", funs, ctx).await?; // 迭代负责人
    let proj_admin_role_id = get_role_id("app_admin_product", funs, ctx).await?; // 产品负责人
    let proj_normal_role_id = get_role_id("app_normal_product", funs, ctx).await?; // 产品人员
    let develop_normal_role_id = get_role_id("app_normal_develop", funs, ctx).await?; // 开发人员
    let develop_admin_role_id = get_role_id("app_admin_develop", funs, ctx).await?; // 开发负责人
    let test_admin_role_id = get_role_id("app_admin_test", funs, ctx).await?; // 测试负责人
    let test_normal_role_id = get_role_id("app_normal_test", funs, ctx).await?; // 测试人员
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
                ("待处理", FlowSysStateKind::Start),
                ("处理中", FlowSysStateKind::Progress),
                ("待确认", FlowSysStateKind::Progress),
                ("已关闭", FlowSysStateKind::Finish),
                ("已撤销", FlowSysStateKind::Finish),
            ],
            "待处理-处理中-待确认-已关闭-已撤销",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_name: "待处理".to_string(),
                    to_flow_state_name: "处理中".to_string(),
                    name: "立即处理".to_string(),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待处理".to_string(),
                    to_flow_state_name: "已撤销".to_string(),
                    name: "撤销".to_string(),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待处理".to_string(),
                    to_flow_state_name: "待确认".to_string(),
                    name: "处理完成".to_string(),
                    guard_by_his_operators: Some(true),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待处理".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该工单？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待确认".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "确认解决".into(),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待确认".to_string(),
                    to_flow_state_name: "处理中".to_string(),
                    name: "未解决".to_string(),
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
        let mut req_role_ids = vec![];
        if !proj_admin_role_id.is_empty() {
            req_role_ids.push(proj_admin_role_id);
        }
        if !proj_normal_role_id.is_empty() {
            req_role_ids.push(proj_normal_role_id);
        }
        // 需求模板初始化
        FlowModelServ::init_model(
            "REQ",
            vec![
                ("待开始", FlowSysStateKind::Start),
                ("进行中", FlowSysStateKind::Progress),
                ("已完成", FlowSysStateKind::Finish),
                ("已关闭", FlowSysStateKind::Finish),
            ],
            "待开始-进行中-已完成-已关闭",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "开始".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if req_role_ids.is_empty() { None } else { Some(req_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该需求？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if req_role_ids.is_empty() { None } else { Some(req_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if req_role_ids.is_empty() { None } else { Some(req_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该需求？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if req_role_ids.is_empty() { None } else { Some(req_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "重新处理".into(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if req_role_ids.is_empty() { None } else { Some(req_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该需求？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if req_role_ids.is_empty() { None } else { Some(req_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已关闭".to_string(),
                    to_flow_state_name: "待开始".to_string(),
                    name: "激活".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if req_role_ids.is_empty() { None } else { Some(req_role_ids.clone()) },
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
                ("待开始", FlowSysStateKind::Start),
                ("进行中", FlowSysStateKind::Progress),
                ("存在风险", FlowSysStateKind::Progress),
                ("已完成", FlowSysStateKind::Progress),
                ("已关闭", FlowSysStateKind::Finish),
                ("已归档", FlowSysStateKind::Finish),
            ],
            "待开始-进行中-存在风险-已完成-已关闭-已归档",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "开始".to_string(),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该项目？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "存在风险".to_string(),
                    name: "有风险".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认该项目有风险？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该项目？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "正常".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将该项目转正常？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该项目？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "重新处理".to_string(),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该项目？".to_string()),
                    }),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "已归档".to_string(),
                    name: "归档".to_string(),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已关闭".to_string(),
                    to_flow_state_name: "待开始".to_string(),
                    name: "激活".to_string(),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已关闭".to_string(),
                    to_flow_state_name: "已归档".to_string(),
                    name: "归档".to_string(),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已归档".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "重新激活".to_string(),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已归档".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该项目？".to_string()),
                    }),
                    ..Default::default()
                },
            ],
            funs,
            ctx,
        )
        .await?;
    }
    let ms_init_model = FlowModelServ::paginate_items(
        &FlowModelFilterReq {
            basic: RbumBasicFilterReq { ..Default::default() },
            tags: Some(vec!["MS".to_string()]),
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
    if ms_init_model.is_none() {
        let mut ms_role_ids = vec![];
        if !admin_role_id.is_empty() {
            ms_role_ids.push(admin_role_id);
        }
        FlowModelServ::init_model(
            "MS",
            vec![
                ("待开始", FlowSysStateKind::Start),
                ("进行中", FlowSysStateKind::Progress),
                ("存在风险", FlowSysStateKind::Progress),
                ("已完成", FlowSysStateKind::Progress),
                ("已关闭", FlowSysStateKind::Finish),
            ],
            "待开始-进行中-存在风险-已完成-已关闭",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "开始".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ms_role_ids.is_empty() { None } else { Some(ms_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该里程碑？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ms_role_ids.is_empty() { None } else { Some(ms_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ms_role_ids.is_empty() { None } else { Some(ms_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "存在风险".to_string(),
                    name: "有风险".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认该里程碑有风险？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ms_role_ids.is_empty() { None } else { Some(ms_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该里程碑？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ms_role_ids.is_empty() { None } else { Some(ms_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "正常".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将该里程碑转正常？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ms_role_ids.is_empty() { None } else { Some(ms_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ms_role_ids.is_empty() { None } else { Some(ms_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该里程碑？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ms_role_ids.is_empty() { None } else { Some(ms_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "重新处理".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ms_role_ids.is_empty() { None } else { Some(ms_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该里程碑？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ms_role_ids.is_empty() { None } else { Some(ms_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已关闭".to_string(),
                    to_flow_state_name: "待开始".to_string(),
                    name: "激活".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ms_role_ids.is_empty() { None } else { Some(ms_role_ids.clone()) },
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
        let mut iter_role_ids = vec![];
        if !iter_admin_role_id.is_empty() {
            iter_role_ids.push(iter_admin_role_id);
        }
        FlowModelServ::init_model(
            "ITER",
            vec![
                ("待开始", FlowSysStateKind::Start),
                ("进行中", FlowSysStateKind::Progress),
                ("存在风险", FlowSysStateKind::Progress),
                ("已完成", FlowSysStateKind::Progress),
                ("已关闭", FlowSysStateKind::Finish),
            ],
            "待开始-进行中-存在风险-已完成-已关闭",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "开始".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if iter_role_ids.is_empty() { None } else { Some(iter_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该迭代？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if iter_role_ids.is_empty() { None } else { Some(iter_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if iter_role_ids.is_empty() { None } else { Some(iter_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "存在风险".to_string(),
                    name: "有风险".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认该迭代有风险？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if iter_role_ids.is_empty() { None } else { Some(iter_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该迭代？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if iter_role_ids.is_empty() { None } else { Some(iter_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "正常".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将该迭代转正常？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if iter_role_ids.is_empty() { None } else { Some(iter_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if iter_role_ids.is_empty() { None } else { Some(iter_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该迭代？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if iter_role_ids.is_empty() { None } else { Some(iter_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "重新处理".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if iter_role_ids.is_empty() { None } else { Some(iter_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该迭代？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if iter_role_ids.is_empty() { None } else { Some(iter_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已关闭".to_string(),
                    to_flow_state_name: "待开始".to_string(),
                    name: "激活".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if iter_role_ids.is_empty() { None } else { Some(iter_role_ids.clone()) },
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
        let mut task_role_ids = vec![];
        if !develop_admin_role_id.is_empty() {
            task_role_ids.push(develop_admin_role_id.clone());
        }
        if !develop_normal_role_id.is_empty() {
            task_role_ids.push(develop_normal_role_id);
        }
        FlowModelServ::init_model(
            "TASK",
            vec![
                ("待开始", FlowSysStateKind::Start),
                ("进行中", FlowSysStateKind::Progress),
                ("存在风险", FlowSysStateKind::Progress),
                ("已完成", FlowSysStateKind::Progress),
                ("已关闭", FlowSysStateKind::Finish),
            ],
            "待开始-进行中-存在风险-已完成-已关闭",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "开始".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if task_role_ids.is_empty() { None } else { Some(task_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该任务？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if task_role_ids.is_empty() { None } else { Some(task_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if task_role_ids.is_empty() { None } else { Some(task_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "存在风险".to_string(),
                    name: "有风险".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认该任务有风险？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if task_role_ids.is_empty() { None } else { Some(task_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该任务？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if task_role_ids.is_empty() { None } else { Some(task_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "正常".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将该任务转正常？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if task_role_ids.is_empty() { None } else { Some(task_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if task_role_ids.is_empty() { None } else { Some(task_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该任务？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if task_role_ids.is_empty() { None } else { Some(task_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "重新处理".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if task_role_ids.is_empty() { None } else { Some(task_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该任务？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if task_role_ids.is_empty() { None } else { Some(task_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已关闭".to_string(),
                    to_flow_state_name: "待开始".to_string(),
                    name: "激活".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if task_role_ids.is_empty() { None } else { Some(task_role_ids.clone()) },
                    ..Default::default()
                },
            ],
            funs,
            ctx,
        )
        .await?;
    }
    let tp_init_model = FlowModelServ::paginate_items(
        &FlowModelFilterReq {
            basic: RbumBasicFilterReq { ..Default::default() },
            tags: Some(vec!["TP".to_string()]),
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
    if tp_init_model.is_none() {
        let mut tp_role_ids = vec![];
        if !test_admin_role_id.is_empty() {
            tp_role_ids.push(test_admin_role_id.clone());
        }
        FlowModelServ::init_model(
            "TP",
            vec![
                ("待开始", FlowSysStateKind::Start),
                ("进行中", FlowSysStateKind::Progress),
                ("存在风险", FlowSysStateKind::Progress),
                ("已完成", FlowSysStateKind::Progress),
                ("已关闭", FlowSysStateKind::Finish),
            ],
            "待开始-进行中-存在风险-已完成-已关闭",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "开始".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if tp_role_ids.is_empty() { None } else { Some(tp_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该测试计划？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if tp_role_ids.is_empty() { None } else { Some(tp_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if tp_role_ids.is_empty() { None } else { Some(tp_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "存在风险".to_string(),
                    name: "有风险".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认该测试计划有风险？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if tp_role_ids.is_empty() { None } else { Some(tp_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该测试计划？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if tp_role_ids.is_empty() { None } else { Some(tp_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "正常".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将该测试计划转正常？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if tp_role_ids.is_empty() { None } else { Some(tp_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if tp_role_ids.is_empty() { None } else { Some(tp_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该测试计划？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if tp_role_ids.is_empty() { None } else { Some(tp_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "重新处理".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if tp_role_ids.is_empty() { None } else { Some(tp_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该测试计划？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if tp_role_ids.is_empty() { None } else { Some(tp_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已关闭".to_string(),
                    to_flow_state_name: "待开始".to_string(),
                    name: "激活".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if tp_role_ids.is_empty() { None } else { Some(tp_role_ids.clone()) },
                    ..Default::default()
                },
            ],
            funs,
            ctx,
        )
        .await?;
    }
    let ts_init_model = FlowModelServ::paginate_items(
        &FlowModelFilterReq {
            basic: RbumBasicFilterReq { ..Default::default() },
            tags: Some(vec!["TS".to_string()]),
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
    if ts_init_model.is_none() {
        let mut ts_role_ids = vec![];
        if !test_admin_role_id.is_empty() {
            ts_role_ids.push(test_admin_role_id.clone());
        }
        if !test_normal_role_id.is_empty() {
            ts_role_ids.push(test_normal_role_id.clone());
        }
        FlowModelServ::init_model(
            "TS",
            vec![
                ("待开始", FlowSysStateKind::Start),
                ("进行中", FlowSysStateKind::Progress),
                ("存在风险", FlowSysStateKind::Progress),
                ("已完成", FlowSysStateKind::Progress),
                ("已关闭", FlowSysStateKind::Finish),
            ],
            "待开始-进行中-存在风险-已完成-已关闭",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "开始".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ts_role_ids.is_empty() { None } else { Some(ts_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待开始".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该测试阶段？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ts_role_ids.is_empty() { None } else { Some(ts_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ts_role_ids.is_empty() { None } else { Some(ts_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "存在风险".to_string(),
                    name: "有风险".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认该测试阶段有风险？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ts_role_ids.is_empty() { None } else { Some(ts_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "进行中".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该测试阶段？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ts_role_ids.is_empty() { None } else { Some(ts_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "正常".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认将该测试阶段转正常？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ts_role_ids.is_empty() { None } else { Some(ts_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已完成".to_string(),
                    name: "完成".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ts_role_ids.is_empty() { None } else { Some(ts_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "存在风险".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该测试阶段？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ts_role_ids.is_empty() { None } else { Some(ts_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "进行中".to_string(),
                    name: "重新处理".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ts_role_ids.is_empty() { None } else { Some(ts_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已完成".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认关闭该测试阶段？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ts_role_ids.is_empty() { None } else { Some(ts_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已关闭".to_string(),
                    to_flow_state_name: "待开始".to_string(),
                    name: "激活".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if ts_role_ids.is_empty() { None } else { Some(ts_role_ids.clone()) },
                    ..Default::default()
                },
            ],
            funs,
            ctx,
        )
        .await?;
    }
    let issue_init_model = FlowModelServ::paginate_items(
        &FlowModelFilterReq {
            basic: RbumBasicFilterReq { ..Default::default() },
            tags: Some(vec!["ISSUE".to_string()]),
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
    if issue_init_model.is_none() {
        let mut issue_role_ids = vec![];
        if !test_admin_role_id.is_empty() {
            issue_role_ids.push(test_admin_role_id.clone());
        }
        if !test_normal_role_id.is_empty() {
            issue_role_ids.push(test_normal_role_id.clone());
        }
        FlowModelServ::init_model(
            "ISSUE",
            vec![
                ("待处理", FlowSysStateKind::Start),
                ("修复中", FlowSysStateKind::Progress),
                ("待确认", FlowSysStateKind::Progress),
                ("已解决", FlowSysStateKind::Progress),
                ("已关闭", FlowSysStateKind::Finish),
            ],
            "待处理-修复中-待确认-已解决-已关闭",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_name: "待处理".to_string(),
                    to_flow_state_name: "修复中".to_string(),
                    name: "确认并修复".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if issue_role_ids.is_empty() { None } else { Some(issue_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待处理".to_string(),
                    to_flow_state_name: "待确认".to_string(),
                    name: "修复完成".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if issue_role_ids.is_empty() { None } else { Some(issue_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "修复中".to_string(),
                    to_flow_state_name: "待确认".to_string(),
                    name: "修复完成".to_string(),
                    guard_by_assigned: Some(true),
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待确认".to_string(),
                    to_flow_state_name: "已解决".to_string(),
                    name: "确认修复".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if issue_role_ids.is_empty() { None } else { Some(issue_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待确认".to_string(),
                    to_flow_state_name: "修复中".to_string(),
                    name: "未修复".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if issue_role_ids.is_empty() { None } else { Some(issue_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已解决".to_string(),
                    to_flow_state_name: "待处理".to_string(),
                    name: "激活".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if issue_role_ids.is_empty() { None } else { Some(issue_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已解决".to_string(),
                    to_flow_state_name: "已关闭".to_string(),
                    name: "关闭".to_string(),

                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if issue_role_ids.is_empty() { None } else { Some(issue_role_ids.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已关闭".to_string(),
                    to_flow_state_name: "待处理".to_string(),
                    name: "激活".to_string(),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if issue_role_ids.is_empty() { None } else { Some(issue_role_ids.clone()) },
                    ..Default::default()
                },
            ],
            funs,
            ctx,
        )
        .await?;
    }
    let cts_init_model = FlowModelServ::paginate_items(
        &FlowModelFilterReq {
            basic: RbumBasicFilterReq { ..Default::default() },
            tags: Some(vec!["CTS".to_string()]),
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
    if cts_init_model.is_none() {
        let mut cts_role_ids1 = vec![];
        if !test_admin_role_id.is_empty() {
            cts_role_ids1.push(test_admin_role_id.clone());
        }
        let mut cts_role_ids2 = vec![];
        if !develop_admin_role_id.is_empty() {
            cts_role_ids2.push(develop_admin_role_id.clone());
        }

        FlowModelServ::init_model(
            "CTS",
            vec![
                ("待接收", FlowSysStateKind::Start),
                ("已接收", FlowSysStateKind::Progress),
                ("已退回", FlowSysStateKind::Finish),
                ("已撤销", FlowSysStateKind::Finish),
            ],
            "待接收-已接收-已退回-已撤销",
            vec![
                FlowTransitionInitInfo {
                    from_flow_state_name: "待接收".to_string(),
                    to_flow_state_name: "已接收".to_string(),
                    name: "接收".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认接收该转测单？".to_string()),
                    }),
                    guard_by_spec_role_ids: if cts_role_ids1.is_empty() { None } else { Some(cts_role_ids1.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待接收".to_string(),
                    to_flow_state_name: "已撤销".to_string(),
                    name: "撤销".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认撤销该转测单？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if cts_role_ids2.is_empty() { None } else { Some(cts_role_ids2.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "待接收".to_string(),
                    to_flow_state_name: "已退回".to_string(),
                    name: "退回".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认退回该转测单？".to_string()),
                    }),
                    guard_by_spec_role_ids: if cts_role_ids1.is_empty() { None } else { Some(cts_role_ids1.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已退回".to_string(),
                    to_flow_state_name: "已接收".to_string(),
                    name: "重新提交".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认重新提交该转测单？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if cts_role_ids2.is_empty() { None } else { Some(cts_role_ids2.clone()) },
                    ..Default::default()
                },
                FlowTransitionInitInfo {
                    from_flow_state_name: "已撤销".to_string(),
                    to_flow_state_name: "待接收".to_string(),
                    name: "重新提交".to_string(),
                    double_check: Some(FlowTransitionDoubleCheckInfo {
                        is_open: true,
                        content: Some("确认重新提交该转测单？".to_string()),
                    }),
                    guard_by_creator: Some(true),
                    guard_by_spec_role_ids: if cts_role_ids2.is_empty() { None } else { Some(cts_role_ids2.clone()) },
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
