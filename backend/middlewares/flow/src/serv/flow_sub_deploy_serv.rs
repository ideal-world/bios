use std::collections::HashMap;

use bios_basic::rbum::{dto::rbum_filer_dto::RbumBasicFilterReq, serv::rbum_item_serv::RbumItemCrudOperation};
use itertools::Itertools;
use serde_json::json;
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult}, chrono::{DateTime, Utc}, db::sea_orm::Set, futures::future::join_all, TardisFuns, TardisFunsInst
};

use crate::{domain::flow_inst, dto::{flow_inst_dto::FlowInstFilterReq, flow_model_dto::{FlowModelAddReq, FlowModelFilterReq}, flow_state_dto::{FlowStateAddReq, FlowStateFilterReq}, flow_sub_deploy_dto::{FlowSubDeployOneExportAggResp, FlowSubDeployOneImportReq, FlowSubDeployTowExportAggResp, FlowSubDeployTowImportReq}}};

use super::{flow_inst_serv::FlowInstServ, flow_model_serv::FlowModelServ, flow_state_serv::FlowStateServ};

pub struct FlowSubDeployServ;

impl FlowSubDeployServ {
    pub(crate) async fn one_deploy_export(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<FlowSubDeployOneExportAggResp> {
        let mut states = HashMap::new();
        let mut models = HashMap::new();
        for model in FlowModelServ::find_detail_items(&FlowModelFilterReq {
            basic: RbumBasicFilterReq {
                own_paths: Some("".to_string()),
                with_sub_own_paths: true,
                ..Default::default()
            },
            data_source: Some(id.to_string()),
            ..Default::default()
        }, Some(true), None, funs, ctx).await? {
            if models.contains_key(&model.tag) {
                continue;
            }
            let model_states = FlowStateServ::find_detail_items(&FlowStateFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(model.states().into_iter().map(|state| state.id).collect_vec()),
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            }, None, None, funs, ctx).await?;
            for model_state in model_states {
                if !states.contains_key(&model_state.id) {
                    states.insert(model_state.id.clone(), model_state);
                }
            }

            models.insert(model.tag.clone(), model);
        }
        Ok(FlowSubDeployOneExportAggResp {
            states: states.values().cloned().collect_vec(),
            models: models.values().cloned().collect_vec(),
        })
    }

    pub(crate) async fn sub_deploy_import(import_req: FlowSubDeployTowImportReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        for import_state in import_req.states {
            let mock_ctx = TardisContext {
                own_paths: import_state.own_paths,
                owner: import_state.owner,
                ..Default::default()
            };
            if FlowStateServ::get_item(&import_state.id, &FlowStateFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            }, funs, &mock_ctx).await.is_ok() {
                continue;
            }
            let mut add_req: FlowStateAddReq = import_state.clone().into();
            add_req.id = Some(TrimString::from(import_state.id.clone()));
            FlowStateServ::add_item(&mut add_req, funs, &mock_ctx).await?;
        }
        for model in import_req.models {
            let mock_ctx = TardisContext {
                own_paths: import_state.own_paths,
                owner: import_state.owner,
                ..Default::default()
            };
            if FlowModelServ::get_item(&model.id, &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            }, funs, &mock_ctx).await.is_ok() {
                continue;
            }
            let mut add_req = model.clone_model();
            FlowModelServ::add_item(&mut add_req, funs, &mock_ctx).await?;
        }
        Ok(())
    }

    pub(crate) async fn sub_deploy_export(
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<FlowSubDeployTowExportAggResp> {
        let insts = FlowInstServ::find_detail_items(&FlowInstFilterReq {
            with_sub: Some(true),
            create_time_start: Some(start_time),
            create_time_end: Some(end_time),
            ..Default::default()
        }, funs, ctx).await?;
        Ok(FlowSubDeployTowExportAggResp {
            insts, 
        })
    }

    pub(crate) async fn one_deploy_import(import_req: FlowSubDeployOneImportReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        join_all(
            import_req.insts
                .iter()
                .map(|inst| async {
                    let flow_inst: flow_inst::ActiveModel = flow_inst::ActiveModel {
                        id: Set(inst.id.clone()),
                        code: Set(Some(inst.code.clone())),
                        tag: Set(Some(inst.tag.clone())),
                        rel_flow_version_id: Set(inst.rel_flow_version_id.clone()),
                        rel_business_obj_id: Set(inst.rel_business_obj_id.clone()),
                        rel_transition_id: Set(inst.rel_transition_id.clone()),
            
                        current_state_id: Set(inst.current_state_id.clone()),
            
                        create_vars: Set(inst.create_vars.clone().map(|vars| TardisFuns::json.obj_to_json(&vars).unwrap_or(json!({})))),
                        current_vars: Set(inst.current_vars.clone().map(|vars| TardisFuns::json.obj_to_json(&vars).unwrap_or(json!({})))),
            
                        create_ctx: Set(inst.create_ctx.clone()),

                        finish_ctx: Set(inst.finish_ctx.clone()),
                        finish_time: Set(inst.finish_time),
                        finish_abort: Set(inst.finish_abort),
                        output_message: Set(inst.output_message.clone()),
                        
                        transitions: Set(inst.transitions.clone()),
                        artifacts: Set(inst.artifacts.clone()),
                        comments: Set(inst.comments.clone()),

                        own_paths: Set(inst.own_paths.clone()),
                        main: Set(inst.main),
                        rel_inst_id: Set(inst.rel_inst_id.clone()),
                        data_source: Set(inst.data_source.clone()),

                        create_time: Set(inst.create_time.clone()),
                        update_time: Set(inst.update_time.clone()),
                    };
                    funs.db().insert_one(flow_inst, ctx).await
                })
                .collect_vec(),
        )
        .await
        .into_iter()
        .collect::<TardisResult<Vec<_>>>()?;
        Ok(())
    }
    
}
