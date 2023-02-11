use crate::basic::dto::iam_cert_dto::{IamCertAkSkAddReq, IamCertAkSkResp};
use crate::basic::serv::iam_cert_aksk_serv::IamCertAkSkServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_enumeration::IamCertKernelKind;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::{TardisFuns, TardisFunsInst};

pub struct IamCiCertAkSkServ;

impl IamCiCertAkSkServ {
    pub async fn general_cert(add_req: IamCertAkSkAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamCertAkSkResp> {
        let cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::AkSk.to_string().as_str(), Some(IamTenantServ::get_id_by_ctx(ctx, funs)?), funs).await?;
        let ak = TardisFuns::crypto.key.generate_ak()?;
        let sk = TardisFuns::crypto.key.generate_sk(&ak)?;

        let cert_id = IamCertAkSkServ::add_cert(&add_req, &ak, &sk, &cert_conf_id, funs, ctx).await?;
        Ok(IamCertAkSkResp { id: cert_id, ak, sk })
    }

    pub async fn delete_cert(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamCertAkSkServ::delete_cert(id, funs, ctx).await
    }
}
