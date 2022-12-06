use tardis::basic::result::TardisResult;
use crate::iam_enumeration::Oauth2GrantType;

pub struct IamCiOauth2AkSkServ;

impl IamCiOauth2AkSkServ {
    pub async fn generate_token(grant_type: Oauth2GrantType, client_id: String, client_secret: String) ->TardisResult<()>{
        //todo
        Ok(())
    }
}