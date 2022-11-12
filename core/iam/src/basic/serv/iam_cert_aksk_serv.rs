use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    TardisFunsInst,
};

use crate::basic::dto::iam_cert_conf_dto::IamCertConfAkSkAddOrModifyReq;

pub struct IamCertAkSkServ;

impl IamCertAkSkServ {
    pub async fn add_cert_conf(add_req: &IamCertConfAkSkAddOrModifyReq, rel_iam_item_id: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        Ok("//todo".into())
    }
}
