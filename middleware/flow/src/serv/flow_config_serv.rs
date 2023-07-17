use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::Utc,
    db::sea_orm::{Set, sea_query::{Query, Expr}, Order},
    TardisFuns, TardisFunsInst,
};

use crate::{dto::flow_config_dto::{FlowConfigModifyReq, FlowConfigAddReq, FlowConfigSummaryResp}, domain::flow_config};

pub struct FlowConfigServ;

impl FlowConfigServ {
    pub async fn add(add_req: &FlowConfigAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let flow_config = flow_config::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            code: Set(add_req.code.to_string()),
            name: Set(add_req.name.to_string()),
            value: Set(add_req.value.to_string()),
            ..Default::default()
        };
        funs.db().insert_one(flow_config, ctx).await?;
        Ok(())
    }

    pub async fn modify(modify_req: &Vec<FlowConfigModifyReq>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let configs = Self::get_config(funs, ctx).await?;
        for req in modify_req {
            if let Some(config) = configs.iter().find(|config|config.code == req.code.clone()) {
                let mut flow_config = flow_config::ActiveModel {
                    id: Set(config.id.to_string()),
                    ..Default::default()
                };
    
                flow_config.value = Set(req.value.to_string());
    
                flow_config.update_time = Set(Utc::now());
                funs.db().update_one(flow_config, ctx).await?;
            }
        }
        Ok(())
    }

    pub async fn get_config(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<FlowConfigSummaryResp>> {
        let mut query = Query::select();
        query
            .columns([
                (flow_config::Entity, flow_config::Column::Id),
                (flow_config::Entity, flow_config::Column::Code),
                (flow_config::Entity, flow_config::Column::Name),
                (flow_config::Entity, flow_config::Column::Value),
                (flow_config::Entity, flow_config::Column::CreateTime),
                (flow_config::Entity, flow_config::Column::UpdateTime),
            ])
            .from(flow_config::Entity)
            .and_where(Expr::col((flow_config::Entity, flow_config::Column::OwnPaths)).eq(ctx.own_paths.clone()))
            .order_by((flow_config::Entity, flow_config::Column::CreateTime), Order::Asc);
        let flow_configs: Vec<FlowConfigSummaryResp> = funs.db().find_dtos(&query).await?;
        Ok(flow_configs)
    }
}