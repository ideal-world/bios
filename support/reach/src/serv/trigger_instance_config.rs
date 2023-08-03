use crate::domain::{trigger_instance_config, trigger_scene};
use crate::dto::*;
use bios_basic::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use tardis::async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::sea_query::{Query, SelectStatement};
use tardis::db::sea_orm::*;
use tardis::{TardisFuns, TardisFunsInst};

pub struct ReachTriggerInstanceConfigService;

#[async_trait]
impl
    RbumCrudOperation<
        trigger_instance_config::ActiveModel,
        ReachTriggerInstanceConfigAddReq,
        ReachTriggerInstanceConfigModifyReq,
        ReachTriggerInstanceConfigSummaryResp,
        ReachTriggerInstanceConfigDetailResp,
        ReachTriggerInstanceConfigFilterReq,
    > for ReachTriggerInstanceConfigService
{
    fn get_table_name() -> &'static str {
        trigger_instance_config::Entity.table_name()
    }
    async fn package_add(add_req: &ReachTriggerInstanceConfigAddReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<trigger_instance_config::ActiveModel> {
        let mut model = trigger_instance_config::ActiveModel::from(add_req);
        model.id = Set(TardisFuns::field.nanoid());
        model.fill_ctx(ctx, true);
        Ok(model)
    }

    async fn before_add_rbum(add_req: &mut ReachTriggerInstanceConfigAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if 1 != funs
            .db()
            .count(Query::select().column(trigger_scene::Column::Id).from(trigger_scene::Entity).and_where(trigger_scene::Column::Id.eq(&add_req.rel_reach_trigger_scene_id)))
            .await?
        {
            return Err(funs.err().bad_request(Self::get_table_name(), "before_add_rbum", "rel_reach_trigger_scene_id is exist", ""));
        }
        let mut filter = ReachTriggerInstanceConfigFilterReq {
            rel_reach_trigger_scene_id: Some(add_req.rel_reach_trigger_scene_id.clone()),
            rel_reach_channel: Some(add_req.rel_reach_channel),
            receive_group_code: vec![add_req.receive_group_code.clone()],
            rel_item_id: Some(add_req.rel_item_id.clone()),
            ..Default::default()
        };
        filter.base_filter.basic.with_sub_own_paths = true;
        if 0 != Self::count_rbums(&filter, funs, ctx).await? {
            return Err(funs.err().bad_request(Self::get_table_name(), "before_add_rbum", "group code is exist", ""));
        }
        Ok(())
    }

    async fn package_modify(
        id: &str,
        modify_req: &ReachTriggerInstanceConfigModifyReq,
        _: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<trigger_instance_config::ActiveModel> {
        let mut model = trigger_instance_config::ActiveModel::from(modify_req);
        model.id = Set(id.into());
        model.fill_ctx(ctx, true);
        Ok(model)
    }

    async fn package_query(is_detail: bool, filter: &ReachTriggerInstanceConfigFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query.from(trigger_instance_config::Entity);
        query
            .columns(trigger_instance_config::Column::iter())
            .and_where_option(filter.rel_reach_trigger_scene_id.as_ref().map(|v| trigger_instance_config::Column::RelReachTriggerSceneId.eq(v)))
            .and_where_option(filter.rel_reach_channel.map(|v| trigger_instance_config::Column::RelReachChannel.eq(v)))
            .and_where_option(filter.rel_item_id.as_ref().map(|v| trigger_instance_config::Column::RelItemId.eq(v)))
            .and_where(trigger_instance_config::Column::ReceiveGroupCode.is_in(&filter.receive_group_code));
        query.with_filter(Self::get_table_name(), &filter.base_filter.basic, is_detail, false, ctx);
        Ok(query)
    }
}

impl ReachTriggerInstanceConfigService {
    async fn add_or_modify_by_single_req(req: ReachTriggerInstanceConfigAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let mut filter = ReachTriggerInstanceConfigFilterReq::default();
        filter.base_filter.basic.with_sub_own_paths = true;
        filter.rel_reach_channel = Some(req.rel_reach_channel);
        filter.rel_reach_trigger_scene_id = Some(req.rel_reach_trigger_scene_id.clone());
        filter.receive_group_code = vec![req.receive_group_code.clone()];
        if let Some(trigger_instance_config) = Self::find_one_rbum(&filter, funs, ctx).await? {
            if req.delete_kind {
                Self::delete_rbum(&trigger_instance_config.id, funs, ctx).await?;
            }
        } else {
            Self::add_rbum(&mut req.into(), funs, ctx).await?;
        }
        Ok(())
    }
    pub async fn add_or_modify_instance_config(req: ReachTriggerInstanceConfigAddOrModifyAggReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        for req in req.instance_config {
            Self::add_or_modify_by_single_req(req, funs, ctx).await?;
        }
        Ok(())
    }
}
