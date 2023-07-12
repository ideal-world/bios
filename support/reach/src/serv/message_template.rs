// package tech.starsys.reach.service;

// import com.ecfront.dew.common.Resp;
// import com.github.yulichang.query.MPJQueryWrapper;
// import group.idealworld.dew.core.basic.resp.StandardResp;
// import tech.starsys.common.rbum.service.RbumCrudQueryPackage;
// import tech.starsys.common.rbum.service.RbumCrudService;
// import tech.starsys.reach.domain.ReachMsgTemplate;
// import tech.starsys.reach.dto.ReachMsgTemplateDto;
// import tech.starsys.reach.enumeration.ReachTemplateKind;
// import tech.starsys.reach.mapper.ReachMsgTemplateMapper;

// import javax.annotation.Resource;
// import javax.inject.Named;
// import java.util.Optional;

// @Named
// public class ReachMsgTemplateService extends RbumCrudService<ReachMsgTemplateMapper, ReachMsgTemplate,
//         ReachMsgTemplateDto.ReachMsgTemplateAddReq,
//         ReachMsgTemplateDto.ReachMsgTemplateModifyReq, ReachMsgTemplateDto.ReachMsgTemplateSummaryResp,
//         ReachMsgTemplateDto.ReachMsgTemplateDetailResp, ReachMsgTemplateDto.ReachMsgTemplateFilterReq> {

//     @Resource
//     private ReachVCodeStrategyService reachVCodeStrategyService;

//     @Override
//     public Resp<Void> beforeAddRbum(ReachMsgTemplateDto.ReachMsgTemplateAddReq addReq) {
//         if (addReq.getKind().equals(ReachTemplateKind.VCODE)) {
// //            var checkResp = reachVCodeStrategyService.checkScope(addReq.getRelReachVerifyCodeStrategyId());
// //            if (!checkResp.ok()) {
// //                return StandardResp.error(checkResp);
// //            }
//         }
//         return super.beforeAddRbum(addReq);
//     }

//     @Override
//     public Resp<MPJQueryWrapper<ReachMsgTemplate>> packageQuery(boolean isDetail, ReachMsgTemplateDto.ReachMsgTemplateFilterReq filter) {
//         MPJQueryWrapper<ReachMsgTemplate> wrapper = new MPJQueryWrapper<>();
//         wrapper.selectAll(getEntityClazz());
//         if (Optional.ofNullable(filter.getRelReachChannel()).isPresent()) {
//             wrapper.eq("t.rel_reach_channel", filter.getRelReachChannel().toString());
//         }
//         if (Optional.ofNullable(filter.getLevelKind()).isPresent()) {
//             wrapper.eq("t.level_kind", filter.getLevelKind().toString());
//         }
//         if (Optional.ofNullable(filter.getKind()).isPresent()) {
//             wrapper.eq("t.kind", filter.getKind().toString());
//         }
//         RbumCrudQueryPackage.withFilter("t", wrapper, filter.getBasic(), isDetail, true);
//         return Resp.success(wrapper);
//     }
// }

use crate::domain::reach_message_template;

use crate::dto::*;

use tardis::async_trait::async_trait;

use bios_basic::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::sea_query::{Query, SelectStatement};
use tardis::db::sea_orm::EntityName;
use tardis::db::sea_orm::{ColumnTrait, Set};
use tardis::TardisFunsInst;

pub struct ReachMessageTemplateServ;

#[async_trait]
impl
    RbumCrudOperation<
        reach_message_template::ActiveModel,
        ReachMessageTemplateAddReq,
        ReachMessageTemplateModifyReq,
        ReachMessageTemplateSummaryResp,
        ReachMessageTemplateDetailResp,
        ReachMessageTemplateFilterReq,
    > for ReachMessageTemplateServ
{
    fn get_table_name() -> &'static str {
        reach_message_template::Entity.table_name()
    }
    async fn package_add(add_req: &ReachMessageTemplateAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<reach_message_template::ActiveModel> {
        let mut model = reach_message_template::ActiveModel::from(add_req);
        model.fill_ctx(ctx, true);
        Ok(model)
    }
    async fn before_add_rbum(add_req: &mut ReachMessageTemplateAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    async fn package_modify(id: &str, modify_req: &ReachMessageTemplateModifyReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<reach_message_template::ActiveModel> {
        let mut model = reach_message_template::ActiveModel::from(modify_req);
        model.id = Set(id.to_string());
        model.fill_ctx(ctx, true);
        Ok(model)
    }

    async fn package_query(is_detail: bool, filter: &ReachMessageTemplateFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        if let Some(chan) = filter.rel_reach_channel {
            query.and_where(reach_message_template::Column::RelReachChannel.eq(chan));
        }
        if let Some(level_kind) = filter.level_kind {
            query.and_where(reach_message_template::Column::LevelKind.eq(level_kind));
        }
        if let Some(kind) = filter.kind {
            query.and_where(reach_message_template::Column::Kind.eq(kind));
        }
        query.with_filter(Self::get_table_name(), &filter.base_filter, is_detail, false, ctx);
        Ok(query)
    }
}

impl ReachMessageTemplateServ {
    pub async fn get_by_id(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<ReachMessageTemplateDetailResp> {
        let rbum = Self::get_rbum(id, &ReachMessageTemplateFilterReq::default(), funs, ctx).await?;
        Ok(rbum)
    }
}
