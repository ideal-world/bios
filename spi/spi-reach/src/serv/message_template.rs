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

use tardis::basic::dto::TardisContext;
use bios_basic::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm::sea_query::{Query, SelectStatement};
use tardis::db::sea_orm::EntityName;
use tardis::db::sea_orm::{ColumnTrait, Set};
use tardis::TardisFunsInst;

pub struct ReachMessageTemplateServ;

#[async_trait]
impl RbumCrudOperation<
    reach_message_template::ActiveModel,
    ReachMessageTemplateAddReq,
    ReachMessageTemplateModifyReq,
    ReachMessageTemplateSummaryResp,
    ReachMessageTemplateDetailResp,
    ReachMessageTemplateFilterReq,
> for ReachMessageTemplateServ {
    
}