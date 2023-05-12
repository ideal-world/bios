use tardis::basic::result::TardisResult;
use tardis::mail::mail_client::{TardisMailClient, TardisMailSendReq};
use tardis::TardisFunsInst;

use crate::iam_config::IamConfig;

pub struct MailClient;

impl MailClient {
    pub async fn send_cert_activate_vcode(mail: &str, account_name: Option<String>, vcode: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        let mut subject = funs.conf::<IamConfig>().mail_template_cert_activate_title.clone();
        let mut content = funs.conf::<IamConfig>().mail_template_cert_activate_content.clone();
        if let Some(account_name) = account_name {
            subject = subject.replace("{account_name}", &account_name);
            content = content.replace("{account_name}", &account_name);
        }
        subject = subject.replace("{vcode}", vcode);
        content = content.replace("{vcode}", vcode);
        Self::send_mail(mail, subject, content, funs).await
    }

    pub async fn send_vcode(mail: &str, account_name: Option<String>, vcode: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        let mut subject = funs.conf::<IamConfig>().mail_template_cert_login_title.clone();
        let mut content = funs.conf::<IamConfig>().mail_template_cert_login_content.clone();
        if let Some(account_name) = account_name {
            subject = subject.replace("{account_name}", &account_name);
            content = content.replace("{account_name}", &account_name);
        }
        subject = subject.replace("{vcode}", vcode);
        content = content.replace("{vcode}", vcode);
        Self::send_mail(mail, subject, content, funs).await
    }

    pub async fn send_pwd(mail: &str, pwd: &str, funs: &TardisFunsInst) -> TardisResult<()> {
        let mut subject = funs.conf::<IamConfig>().mail_template_cert_random_pwd_title.clone();
        let mut content = funs.conf::<IamConfig>().mail_template_cert_random_pwd_content.clone();
        subject = subject.replace("{pwd}", pwd);
        content = content.replace("{pwd}", pwd);
        Self::send_mail(mail, subject, content, funs).await
    }

    pub async fn send_mail(mail: &str, subject: String, content: String, funs: &TardisFunsInst) -> TardisResult<()> {
        TardisMailClient::send_quiet(
            funs.module_code().to_string(),
            TardisMailSendReq {
                subject,
                txt_body: content,
                html_body: None,
                to: vec![mail.to_string()],
                reply_to: None,
                cc: None,
                bcc: None,
                from: None,
            },
        )
    }
}
