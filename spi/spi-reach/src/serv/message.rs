use crate::domain::reach_message;
use crate::dto::*;
use crate::serv::message_signature::ReachMessageSignatureServ;
use bios_basic::rbum::dto::rbum_domain_dto::RbumDomainDetailResp;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use tardis::async_trait::async_trait;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::*;
use tardis::TardisFunsInst;
use tardis::db::sea_orm::sea_query::SelectStatement;
use tardis::{log, TardisFuns};

use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfAddReq, RbumCertConfDetailResp, RbumCertConfIdAndExtResp, RbumCertConfModifyReq, RbumCertConfSummaryResp};
use bios_basic::rbum::dto::rbum_cert_dto::{RbumCertAddReq, RbumCertDetailResp, RbumCertModifyReq, RbumCertSummaryResp};
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumCertConfFilterReq, RbumCertFilterReq};
use bios_basic::rbum::rbum_config::RbumConfigApi;
use bios_basic::rbum::rbum_enumeration::{RbumCertConfStatusKind, RbumCertRelKind, RbumCertStatusKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudQueryPackage;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemServ;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;
use bios_basic::rbum::serv::rbum_set_serv::RbumSetServ;

pub struct ReachMessageServ;
#[async_trait]
impl RbumCrudOperation<reach_message::ActiveModel, ReachMessageAddReq, ReachMessageModifyReq, ReachMessageSummaryResp, ReachMessageDetailResp, ReachMessageFilterReq>
    for ReachMessageServ
{
    fn get_table_name() -> &'static str {
        reach_message::Entity.table_name()
    }
    async fn package_add(add_req: &ReachMessageAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<reach_message::ActiveModel> {
        Ok(reach_message::ActiveModel {
            from_res: Set(add_req.from_res.to_string()),
            rel_reach_channel: Set(add_req.rel_reach_channel.to_string()),
            receive_kind: Set(add_req.receive_kind.to_string()),
            to_res_ids: Set(add_req.to_res_ids.to_string()),
            rel_reach_msg_signature_id: Set(add_req.rel_reach_msg_signature_id.to_string()),
            rel_reach_msg_template_id: Set(add_req.rel_reach_msg_template_id.to_string()),
            content_replace: Set(add_req.rel_reach_msg_template_id.to_string()),
            reach_status: Set(add_req.reach_status),
            ..Default::default()
        })
    }

    async fn before_add_rbum(add_req: &mut ReachMessageAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        ReachMessageSignatureServ::check_ownership(&add_req.rel_reach_msg_signature_id, funs, ctx).await?;
        
        todo!();
    }

    async fn package_modify(id: &str, modify_req: &ReachMessageModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<reach_message::ActiveModel> {
        todo!();
    }

    async fn package_query(is_detail: bool, filter: &ReachMessageFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        todo!();
    }
}
