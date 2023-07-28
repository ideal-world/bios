use std::collections::HashMap;

use tardis::basic::result::TardisResult;

use crate::invoke::Client;
use bios_sdk_invoke::clients::InvokeClient;

impl Client<'_> {
    pub async fn general_send(&self, to: &str, msg_template_id: &str, replacement: HashMap<String, String>) -> TardisResult<()> {
        let url = self.get_url(&["/cc/msg/general", to, msg_template_id], None);
        let header = self.get_tardis_context_header()?;
        let resp = self.funs.web_client().put::<_, ()>(&url, &replacement, Some(vec![header])).await?;
        Self::extract_response(resp)
    }
    pub async fn vcode_send(&self, to: &str, code: &str) -> TardisResult<()> {
        let url = self.get_url(&["/cc/msg/vcode", to, code], None);
        let header = self.get_tardis_context_header()?;
        let resp = self.funs.web_client().put::<_, ()>(&url, &(), Some(vec![header])).await?;
        Self::extract_response(resp)
    }
    pub async fn pwd_send(&self, to: &str, code: &str) -> TardisResult<()> {
        let url = self.get_url(&["/cc/msg/pwd", to, code], None);
        let header = self.get_tardis_context_header()?;
        let resp = self.funs.web_client().put::<_, ()>(&url, &(), Some(vec![header])).await?;
        Self::extract_response(resp)
    }
    pub async fn mail_pwd_send(&self, mail: &str, message: &str, subject: &str) -> TardisResult<()> {
        let url = self.get_url(&["/cc/msg/mail", mail], [("message", message), ("subject", subject)]);
        let header = self.get_tardis_context_header()?;
        let resp = self.funs.web_client().put::<_, ()>(&url, &(), Some(vec![header])).await?;
        Self::extract_response(resp)
    }
}
