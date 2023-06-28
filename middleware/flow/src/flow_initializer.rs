use bios_basic::rbum::{
    dto::{rbum_domain_dto::RbumDomainAddReq, rbum_filer_dto::RbumBasicFilterReq, rbum_kind_dto::RbumKindAddReq},
    rbum_enumeration::RbumScopeLevelKind,
    rbum_initializer,
    serv::{rbum_crud_serv::RbumCrudOperation, rbum_domain_serv::RbumDomainServ, rbum_kind_serv::RbumKindServ},
};
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    db::{reldb_client::TardisActiveModel, sea_orm::sea_query::Table},
    log::info,
    web::web_server::TardisWebServer,
    TardisFuns, TardisFunsInst,
};

use crate::{
    api::cc::{flow_cc_inst_api, flow_cc_model_api, flow_cc_state_api},
    domain::{flow_inst, flow_model, flow_state, flow_transition},
    dto::{flow_state_dto::FlowSysStateKind, flow_transition_dto::FlowTransitionInitInfo, flow_var_dto::{FlowVarInfo, RbumDataTypeKind, RbumWidgetTypeKind}},
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
            (flow_cc_state_api::FlowCcStateApi, flow_cc_model_api::FlowCcModelApi, flow_cc_inst_api::FlowCcInstApi),
            Vec::new(),
        )
        .await;
    Ok(())
}

pub async fn init_db(mut funs: TardisFunsInst) -> TardisResult<()> {
    bios_basic::rbum::rbum_initializer::init(funs.module_code(), funs.conf::<FlowConfig>().rbum.clone()).await?;

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
        funs.db().init(flow_state::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
        funs.db().init(flow_model::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
        funs.db().init(flow_transition::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
        funs.db().init(flow_inst::ActiveModel::init(db_kind, None, compatible_type.clone())).await?;
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

async fn init_model(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
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
        "默认工单模板",
        vec![
            FlowTransitionInitInfo {
                from_flow_state_name: "待处理".to_string(),
                to_flow_state_name: "处理中".to_string(),
                name: "立即处理".to_string(),
                transfer_by_auto: None,
                transfer_by_timer: None,
                guard_by_creator: Some(true),
                guard_by_his_operators: None,
                guard_by_assigned: None,
                guard_by_spec_account_ids: None,
                guard_by_spec_role_ids: None,
                guard_by_other_conds: None,
                vars_collect: None,
                action_by_pre_callback: None,
                action_by_post_callback: None,
            },
            FlowTransitionInitInfo {
                from_flow_state_name: "待处理".to_string(),
                to_flow_state_name: "已撤销".to_string(),
                name: "撤销".to_string(),
                transfer_by_auto: None,
                transfer_by_timer: None,
                guard_by_creator: Some(true),
                guard_by_his_operators: None,
                guard_by_assigned: None,
                guard_by_spec_account_ids: None,
                guard_by_spec_role_ids: None,
                guard_by_other_conds: None,
                vars_collect: None,
                action_by_pre_callback: None,
                action_by_post_callback: None,
            },
            FlowTransitionInitInfo {
                from_flow_state_name: "处理中".to_string(),
                to_flow_state_name: "待确认".to_string(),
                name: "处理完成".to_string(),
                transfer_by_auto: None,
                transfer_by_timer: None,
                guard_by_creator: None,
                guard_by_his_operators: Some(true),
                guard_by_assigned: None,
                guard_by_spec_account_ids: None,
                guard_by_spec_role_ids: None,
                guard_by_other_conds: None,
                vars_collect: None,
                action_by_pre_callback: None,
                action_by_post_callback: None,
            },
            FlowTransitionInitInfo {
                from_flow_state_name: "处理中".to_string(),
                to_flow_state_name: "已关闭".to_string(),
                name: "关闭".to_string(),
                transfer_by_auto: None,
                transfer_by_timer: None,
                guard_by_creator: Some(true),
                guard_by_his_operators: None,
                guard_by_assigned: None,
                guard_by_spec_account_ids: None,
                guard_by_spec_role_ids: None,
                guard_by_other_conds: None,
                vars_collect: None,
                action_by_pre_callback: None,
                action_by_post_callback: None,
            },
            FlowTransitionInitInfo {
                from_flow_state_name: "待确认".to_string(),
                to_flow_state_name: "已关闭".to_string(),
                name: "确认解决".into(),
                transfer_by_auto: None,
                transfer_by_timer: None,
                guard_by_creator: Some(true),
                guard_by_his_operators: None,
                guard_by_assigned: None,
                guard_by_spec_account_ids: None,
                guard_by_spec_role_ids: None,
                guard_by_other_conds: None,
                vars_collect: None,
                action_by_pre_callback: None,
                action_by_post_callback: None,
            },
            FlowTransitionInitInfo {
                from_flow_state_name: "待确认".to_string(),
                to_flow_state_name: "处理中".to_string(),
                name: "未解决".to_string(),
                transfer_by_auto: None,
                transfer_by_timer: None,
                guard_by_creator: Some(true),
                guard_by_his_operators: None,
                guard_by_assigned: None,
                guard_by_spec_account_ids: None,
                guard_by_spec_role_ids: None,
                guard_by_other_conds: None,
                vars_collect: None,
                action_by_pre_callback: None,
                action_by_post_callback: None,
            },
        ],
        funs,
        ctx,
    )
    .await?;
    // 需求模板初始化
    FlowModelServ::init_model(
        "REQ",
        vec![
            ("待开始", FlowSysStateKind::Start),
            ("进行中", FlowSysStateKind::Progress),
            ("已完成", FlowSysStateKind::Finish),
            ("已关闭", FlowSysStateKind::Finish),
        ],
        "默认需求模板",
        vec![
            FlowTransitionInitInfo {
                from_flow_state_name: "待开始".to_string(),
                to_flow_state_name: "进行中".to_string(),
                name: "开始".to_string(),
                transfer_by_auto: None,
                transfer_by_timer: None,
                guard_by_creator: None,
                guard_by_his_operators: None,
                guard_by_assigned: Some(true),
                guard_by_spec_account_ids: None,
                guard_by_spec_role_ids: None,
                guard_by_other_conds: None,
                vars_collect: Some(vec![
                    FlowVarInfo {
                        name: "assigned".to_string(),
                        label: "负责人".to_string(),
                        data_type: RbumDataTypeKind::STRING,
                        widget_type: RbumWidgetTypeKind::SELECT,
                        note: None,
                        sort: None,
                        hide: None,
                        secret: None,
                        show_by_conds: None,
                        widget_columns: None,
                        default_value: None,
                        dyn_default_value: None,
                        options: None,
                        dyn_options: None,
                        required: Some(true),
                        min_length: None,
                        max_length: None,
                        action: None,
                        ext: None,
                        parent_attr_name: None,
                    },
                    FlowVarInfo {
                        name: "start_end".to_string(),
                        label: "计划周期".to_string(),
                        data_type: RbumDataTypeKind::DATETIME,
                        widget_type: RbumWidgetTypeKind::DATETIME,
                        note: None,
                        sort: None,
                        hide: None,
                        secret: None,
                        show_by_conds: None,
                        widget_columns: None,
                        default_value: None,
                        dyn_default_value: None,
                        options: None,
                        dyn_options: None,
                        required: None,
                        min_length: None,
                        max_length: None,
                        action: None,
                        ext: None,
                        parent_attr_name: None,
                    },
                ]),
                action_by_pre_callback: None,
                action_by_post_callback: None,
            },
            FlowTransitionInitInfo {
                from_flow_state_name: "待开始".to_string(),
                to_flow_state_name: "已关闭".to_string(),
                name: "关闭".to_string(),
                transfer_by_auto: None,
                transfer_by_timer: None,
                guard_by_creator: None,
                guard_by_his_operators: None,
                guard_by_assigned: None,
                guard_by_spec_account_ids: None,
                guard_by_spec_role_ids: None,
                guard_by_other_conds: None,
                vars_collect: None,
                action_by_pre_callback: None,
                action_by_post_callback: None,
            },
            FlowTransitionInitInfo {
                from_flow_state_name: "进行中".to_string(),
                to_flow_state_name: "已完成".to_string(),
                name: "完成".to_string(),
                transfer_by_auto: None,
                transfer_by_timer: None,
                guard_by_creator: None,
                guard_by_his_operators: None,
                guard_by_assigned: Some(true),
                guard_by_spec_account_ids: None,
                guard_by_spec_role_ids: None,
                guard_by_other_conds: None,
                vars_collect: Some(vec![
                    FlowVarInfo {
                        name: "assigned".to_string(),
                        label: "负责人".to_string(),
                        data_type: RbumDataTypeKind::STRING,
                        widget_type: RbumWidgetTypeKind::SELECT,
                        note: None,
                        sort: None,
                        hide: None,
                        secret: None,
                        show_by_conds: None,
                        widget_columns: None,
                        default_value: None,
                        dyn_default_value: None,
                        options: None,
                        dyn_options: None,
                        required: Some(true),
                        min_length: None,
                        max_length: None,
                        action: None,
                        ext: None,
                        parent_attr_name: None,
                    },
                    FlowVarInfo {
                        name: "start_end".to_string(),
                        label: "计划周期".to_string(),
                        data_type: RbumDataTypeKind::DATETIME,
                        widget_type: RbumWidgetTypeKind::DATETIME,
                        note: None,
                        sort: None,
                        hide: None,
                        secret: None,
                        show_by_conds: None,
                        widget_columns: None,
                        default_value: None,
                        dyn_default_value: None,
                        options: None,
                        dyn_options: None,
                        required: Some(true),
                        min_length: None,
                        max_length: None,
                        action: None,
                        ext: None,
                        parent_attr_name: None,
                    },
                ]),
                action_by_pre_callback: None,
                action_by_post_callback: None,
            },
            FlowTransitionInitInfo {
                from_flow_state_name: "进行中".to_string(),
                to_flow_state_name: "已关闭".to_string(),
                name: "关闭".to_string(),
                transfer_by_auto: None,
                transfer_by_timer: None,
                guard_by_creator: None,
                guard_by_his_operators: None,
                guard_by_assigned: None,
                guard_by_spec_account_ids: None,
                guard_by_spec_role_ids: None,
                guard_by_other_conds: None,
                vars_collect: None,
                action_by_pre_callback: None,
                action_by_post_callback: None,
            },
            FlowTransitionInitInfo {
                from_flow_state_name: "已完成".to_string(),
                to_flow_state_name: "进行中".to_string(),
                name: "重新处理".into(),
                transfer_by_auto: None,
                transfer_by_timer: None,
                guard_by_creator: None,
                guard_by_his_operators: None,
                guard_by_assigned: Some(true),
                guard_by_spec_account_ids: None,
                guard_by_spec_role_ids: None,
                guard_by_other_conds: None,
                vars_collect: Some(vec![
                    FlowVarInfo {
                        name: "assigned".to_string(),
                        label: "负责人".to_string(),
                        data_type: RbumDataTypeKind::STRING,
                        widget_type: RbumWidgetTypeKind::SELECT,
                        note: None,
                        sort: None,
                        hide: None,
                        secret: None,
                        show_by_conds: None,
                        widget_columns: None,
                        default_value: None,
                        dyn_default_value: None,
                        options: None,
                        dyn_options: None,
                        required: Some(true),
                        min_length: None,
                        max_length: None,
                        action: None,
                        ext: None,
                        parent_attr_name: None,
                    },
                    FlowVarInfo {
                        name: "start_end".to_string(),
                        label: "计划周期".to_string(),
                        data_type: RbumDataTypeKind::DATETIME,
                        widget_type: RbumWidgetTypeKind::DATETIME,
                        note: None,
                        sort: None,
                        hide: None,
                        secret: None,
                        show_by_conds: None,
                        widget_columns: None,
                        default_value: None,
                        dyn_default_value: None,
                        options: None,
                        dyn_options: None,
                        required: None,
                        min_length: None,
                        max_length: None,
                        action: None,
                        ext: None,
                        parent_attr_name: None,
                    },
                ]),
                action_by_pre_callback: None,
                action_by_post_callback: None,
            },
            FlowTransitionInitInfo {
                from_flow_state_name: "已完成".to_string(),
                to_flow_state_name: "已关闭".to_string(),
                name: "关闭".to_string(),
                transfer_by_auto: None,
                transfer_by_timer: None,
                guard_by_creator: None,
                guard_by_his_operators: None,
                guard_by_assigned: None,
                guard_by_spec_account_ids: None,
                guard_by_spec_role_ids: None,
                guard_by_other_conds: None,
                vars_collect: None,
                action_by_pre_callback: None,
                action_by_post_callback: None,
            },
            FlowTransitionInitInfo {
                from_flow_state_name: "已关闭".to_string(),
                to_flow_state_name: "待开始".to_string(),
                name: "激活".to_string(),
                transfer_by_auto: None,
                transfer_by_timer: None,
                guard_by_creator: None,
                guard_by_his_operators: None,
                guard_by_assigned: Some(true),
                guard_by_spec_account_ids: None,
                guard_by_spec_role_ids: None,
                guard_by_other_conds: None,
                vars_collect: None,
                action_by_pre_callback: None,
                action_by_post_callback: None,
            },
        ],
        funs,
        ctx,
    )
    .await?;
    Ok(())
}
