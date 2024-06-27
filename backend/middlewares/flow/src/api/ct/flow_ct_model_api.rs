use std::collections::HashMap;

use bios_basic::rbum::{dto::rbum_filer_dto::RbumBasicFilterReq, serv::rbum_item_serv::RbumItemCrudOperation};
use itertools::Itertools;
use tardis::web::{
    context_extractor::TardisContextExtractor,
    poem::{web::Json, Request},
    poem_openapi::{self, param::Path},
    web_resp::{TardisApiResult, TardisResp, Void},
};

use crate::{
    dto::flow_model_dto::{FlowModelAggResp, FlowModelAssociativeOperationKind, FlowModelCopyOrReferenceReq, FlowModelFilterReq},
    flow_constants,
    serv::{
        flow_model_serv::FlowModelServ,
        flow_rel_serv::{FlowRelKind, FlowRelServ},
    },
};

#[derive(Clone)]
pub struct FlowCtModelApi;

/// Flow model process API
#[poem_openapi::OpenApi(prefix_path = "/ct/model")]
impl FlowCtModelApi {
    /// Creating or referencing models
    ///
    /// 创建或引用模型
    #[oai(path = "/copy_or_reference_model", method = "post")]
    async fn copy_or_reference_model(
        &self,
        req: Json<FlowModelCopyOrReferenceReq>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<HashMap<String, FlowModelAggResp>> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let mut result = HashMap::new();
        let orginal_models = FlowModelServ::find_rel_models(req.0.rel_template_id.clone(), true, &funs, &ctx.0).await?;
        for (tag, rel_model_id) in req.0.rel_model_ids {
            let orginal_model_id = orginal_models.get(&tag).map(|orginal_model| orginal_model.id.clone());

            let added_model = FlowModelServ::copy_or_reference_model(orginal_model_id, &rel_model_id, None, &req.0.op, Some(true), &funs, &ctx.0).await?;
            if let Some(rel_template_id) = &req.0.rel_template_id {
                FlowRelServ::add_simple_rel(
                    &FlowRelKind::FlowModelTemplate,
                    &added_model.id,
                    rel_template_id,
                    None,
                    None,
                    false,
                    true,
                    None,
                    &funs,
                    &ctx.0,
                )
                .await?;
            }
            result.insert(rel_model_id.clone(), added_model);
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Creating or referencing models
    ///
    /// 创建或引用模型
    #[oai(path = "/copy_models_by_template_id/:from_template_id/:to_template_id", method = "post")]
    async fn copy_models_by_template_id(
        &self,
        from_template_id: Path<String>,
        to_template_id: Path<String>,
        ctx: TardisContextExtractor,
        _request: &Request,
    ) -> TardisApiResult<HashMap<String, FlowModelAggResp>> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        let mut result = HashMap::new();
        for from_model in FlowModelServ::find_detail_items(
            &FlowModelFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(
                        FlowRelServ::find_to_simple_rels(&FlowRelKind::FlowModelTemplate, &from_template_id.0, None, None, &funs, &ctx.0)
                            .await?
                            .into_iter()
                            .map(|rel| rel.rel_id)
                            .collect_vec(),
                    ),
                    ignore_scope: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            None,
            &funs,
            &ctx.0,
        )
        .await?
        {
            let added_model =
                FlowModelServ::copy_or_reference_model(None, &from_model.rel_model_id, None, &FlowModelAssociativeOperationKind::Copy, Some(true), &funs, &ctx.0).await?;
            FlowRelServ::add_simple_rel(
                &FlowRelKind::FlowModelTemplate,
                &added_model.id,
                &to_template_id.0,
                None,
                None,
                false,
                true,
                None,
                &funs,
                &ctx.0,
            )
            .await?;
            result.insert(from_model.rel_model_id.clone(), added_model);
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// batch delete models by rel_template_id
    ///
    /// 通过关联模板ID删除模型
    #[oai(path = "/delete_by_rel_template_id/:rel_template_id", method = "delete")]
    async fn delete_by_rel_template_id(&self, rel_template_id: Path<String>, ctx: TardisContextExtractor, _request: &Request) -> TardisApiResult<Void> {
        let mut funs = flow_constants::get_tardis_inst();
        funs.begin().await?;
        for rel in FlowRelServ::find_to_simple_rels(&FlowRelKind::FlowModelTemplate, &rel_template_id.0, None, None, &funs, &ctx.0).await? {
            FlowModelServ::delete_item(&rel.rel_id, &funs, &ctx.0).await?;
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void)
    }
}
