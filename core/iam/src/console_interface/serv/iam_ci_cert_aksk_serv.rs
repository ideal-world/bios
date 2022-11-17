use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::{TardisFuns, TardisFunsInst};
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertAddReq;
use bios_basic::rbum::rbum_enumeration::{RbumCertRelKind, RbumCertStatusKind};
use bios_basic::rbum::serv::rbum_cert_serv::RbumCertServ;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use crate::basic::dto::iam_cert_dto::IamCertAkSkAddReq;
use crate::basic::serv::iam_cert_aksk_serv::IamCertAkSkServ;

pub struct IamCiCertAkSkServ;


impl IamCiCertAkSkServ {
    pub async fn general_cert(app_id: &str, rel_rbum_cert_conf_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let ak = TardisFuns::crypto.key.generate_ak()?;
        let sk = TardisFuns::crypto.key.generate_sk(&ak)?;

        let cert_id = IamCertAkSkServ::add_cert(&IamCertAkSkAddReq { ak, sk }, app_id, rel_rbum_cert_conf_id, funs, ctx).await?;
    }
}
