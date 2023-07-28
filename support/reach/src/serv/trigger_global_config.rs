use crate::domain::{message_signature, message_template, trigger_global_config, trigger_scene};
use crate::dto::*;
use bios_basic::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use tardis::async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;

use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::sea_query::{Query, SelectStatement};
use tardis::db::sea_orm::*;
use tardis::{TardisFunsInst, TardisFuns};



pub struct ReachTriggerGlobalConfigService;

#[async_trait]
impl
    RbumCrudOperation<
        trigger_global_config::ActiveModel,
        ReachTriggerGlobalConfigAddReq,
        ReachTriggerGlobalConfigModifyReq,
        ReachTriggerGlobalConfigSummaryResp,
        ReachTriggerGlobalConfigDetailResp,
        ReachTriggerGlobalConfigFilterReq,
    > for ReachTriggerGlobalConfigService
{
    fn get_table_name() -> &'static str {
        trigger_global_config::Entity.table_name()
    }
    async fn package_add(add_req: &ReachTriggerGlobalConfigAddReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<trigger_global_config::ActiveModel> {
        let mut model = trigger_global_config::ActiveModel::from(add_req);
        model.id = Set(TardisFuns::field.nanoid());
        model.fill_ctx(ctx, true);
        Ok(model)
    }

    async fn before_add_rbum(add_req: &mut ReachTriggerGlobalConfigAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if 1 != funs
            .db()
            .count(Query::select().column(trigger_scene::Column::Id).from(trigger_scene::Entity).and_where(trigger_scene::Column::Id.eq(&add_req.rel_reach_trigger_scene_id)))
            .await?
        {
            return Err(funs.err().bad_request("reach_trigger_global_config", "before_add_rbum", "rel_reach_trigger_scene_id is exist", ""));
        }
        if 1 != funs
            .db()
            .count(
                Query::select()
                    .column(message_signature::Column::Id)
                    .from(message_signature::Entity)
                    .and_where(message_signature::Column::Id.eq(&add_req.rel_reach_msg_signature_id)),
            )
            .await?
        {
            return Err(funs.err().bad_request("reach_trigger_global_config", "before_add_rbum", "rel_reach_msg_signature_id is exist", ""));
        }
        if 1 != funs
            .db()
            .count(
                Query::select().column(message_template::Column::Id).from(message_template::Entity).and_where(message_template::Column::Id.eq(&add_req.rel_reach_msg_template_id)),
            )
            .await?
        {
            return Err(funs.err().bad_request("reach_trigger_global_config", "before_add_rbum", "rel_reach_msg_template_id is exist", ""));
        }
        let mut filter = ReachTriggerGlobalConfigFilterReq {
            rel_reach_trigger_scene_id: Some(add_req.rel_reach_trigger_scene_id.clone()),
            rel_reach_channel: Some(add_req.rel_reach_channel),
            ..Default::default()
        };
        filter.base_filter.basic.with_sub_own_paths = true;

        if 0 != Self::count_rbums(&filter, funs, ctx).await? {
            return Err(funs.err().bad_request("reach_trigger_global_config", "before_add_rbum", "reach_trigger_scene_id and reach_channel is exist", ""));
        }
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &ReachTriggerGlobalConfigModifyReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<trigger_global_config::ActiveModel> {
        let mut model = trigger_global_config::ActiveModel::from(modify_req);
        model.id = Set(id.into());
        model.fill_ctx(ctx, true);
        Ok(model)
    }

    async fn package_query(is_detail: bool, filter: &ReachTriggerGlobalConfigFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(trigger_global_config::Column::iter())
            .and_where_option(filter.rel_reach_trigger_scene_id.as_ref().map(|v| trigger_global_config::Column::RelReachTriggerSceneId.eq(v)))
            .and_where_option(filter.rel_reach_channel.map(|v| trigger_global_config::Column::RelReachChannel.eq(v)))
            .and_where_option(filter.rel_reach_msg_signature_id.as_ref().map(|v| trigger_global_config::Column::RelReachMsgSignatureId.eq(v)))
            .and_where_option(filter.rel_reach_msg_template_id.as_ref().map(|v| trigger_global_config::Column::RelReachMsgTemplateId.eq(v)))
            .and_where(trigger_global_config::Column::RelReachChannel.is_not_in(&filter.not_ids));
        query.with_filter(Self::get_table_name(), &filter.base_filter.basic, is_detail, false, ctx);
        Ok(query)
    }
}

impl ReachTriggerGlobalConfigService {
    async fn add_or_modify_by_single_req(mut req: ReachTriggerGlobalConfigAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let mut filter = ReachTriggerGlobalConfigFilterReq::default();
        filter.base_filter.basic.with_sub_own_paths = true;
        filter.rel_reach_channel = Some(req.rel_reach_channel);
        filter.rel_reach_trigger_scene_id = Some(req.rel_reach_trigger_scene_id.clone());
        if let Some(trigger_global_config) = Self::find_one_rbum(&filter, funs, ctx).await? {
            let mut modify_req = ReachTriggerGlobalConfigModifyReq {
                rel_reach_msg_signature_id: Some(req.rel_reach_msg_signature_id),
                rel_reach_msg_template_id: Some(req.rel_reach_msg_template_id),
                ..Default::default()
            };
            Self::modify_rbum(&trigger_global_config.id, &mut modify_req, funs, ctx).await.map(|_| trigger_global_config.id)
        } else {
            Self::add_rbum(&mut req, funs, ctx).await
        }
    }
    pub async fn add_or_modify_global_config(agg_req: ReachTriggerGlobalConfigAddOrModifyAggReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        for req in agg_req.global_config {
            Self::add_or_modify_by_single_req(req, funs, ctx).await?;
        }
        Ok(())
    }
}
