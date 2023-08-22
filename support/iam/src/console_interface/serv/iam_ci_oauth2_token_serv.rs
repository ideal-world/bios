use crate::basic::dto::iam_cert_dto::IamOauth2AkSkResp;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::iam_enumeration::{IamCertKernelKind, IamCertTokenKind, Oauth2GrantType};
use bios_basic::rbum::rbum_enumeration::RbumCertRelKind;
use tardis::basic::result::TardisResult;
use tardis::{TardisFuns, TardisFunsInst};

pub struct IamCiOauth2AkSkServ;

impl IamCiOauth2AkSkServ {
    pub async fn generate_token(
        grant_type: Oauth2GrantType,
        client_id: &str,
        client_secret: &str,
        _scope: Option<String>,
        funs: TardisFunsInst,
    ) -> TardisResult<IamOauth2AkSkResp> {
        let (_, _, rel_iam_item_id) = IamCertServ::validate_by_ak_and_sk(
            client_id,
            client_secret,
            None,
            Some(&RbumCertRelKind::Item),
            false,
            None,
            Some(vec![&IamCertKernelKind::AkSk.to_string()]),
            None,
            &funs,
        )
        .await?;
        match grant_type {
            Oauth2GrantType::AuthorizationCode => {}
            Oauth2GrantType::Password => {}
            Oauth2GrantType::ClientCredentials => {}
        }

        let access_token = TardisFuns::crypto.key.generate_token()?;
        let refresh_token = TardisFuns::crypto.key.generate_token()?;
        let expire_sec = 30 * 24 * 60 * 60;
        IamIdentCacheServ::add_token(&access_token.clone(), &IamCertTokenKind::TokenOauth2, &rel_iam_item_id, None, expire_sec, 1, &funs).await?;
        Ok(IamOauth2AkSkResp {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: expire_sec.to_string(),
            refresh_token,
            scope: "".to_string(),
        })
    }
}
