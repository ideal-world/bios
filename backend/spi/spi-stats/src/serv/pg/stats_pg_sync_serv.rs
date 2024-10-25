use tardis::{basic::{dto::TardisContext, result::TardisResult}, TardisFunsInst};

use crate::dto::stats_conf_dto::{StatsSyncDbConfigAddReq, StatsSyncDbConfigModifyReq};

pub(crate) async fn db_config_add(add_req: StatsSyncDbConfigAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    // todo 使用rel_rbum_id kind supplier 来作为unique key
    todo!()
}
pub(crate) async fn db_config_modify(modify_req: StatsSyncDbConfigModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    todo!()
}
