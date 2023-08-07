use std::sync::{Arc, OnceLock};

use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use tardis::{TardisFuns, TardisFunsInst};

use crate::{
    client::{email::MailClient, sms::SmsClient},
    config::ReachConfig,
};
pub const DOMAIN_CODE: &str = "reach";
pub const RBUM_KIND_CODE_REACH_MESSAGE: &str = "reach-message";
pub const RBUM_EXT_TABLE_REACH_MESSAGE: &str = "reach_message";
pub const RBUM_SET_SCHEME_REACH: &str = "reach_set_";
pub const MQ_REACH_TOPIC_MESSAGE: &str = "starsys::reach::topic::message";

pub const RBUM_SCOPE_LEVEL_PRIVATE: RbumScopeLevelKind = RbumScopeLevelKind::Private;
pub const RBUM_SCOPE_LEVEL_GLOBAL: RbumScopeLevelKind = RbumScopeLevelKind::Root;
pub const RBUM_SCOPE_LEVEL_TENANT: RbumScopeLevelKind = RbumScopeLevelKind::L1;
pub const RBUM_SCOPE_LEVEL_APP: RbumScopeLevelKind = RbumScopeLevelKind::L2;

pub const REACH_INIT_OWNER: &str = "ReachInit";

pub const IAM_KEY_PHONE_V_CODE: &str = "PhoneVCode";
pub static DOMAIN_REACH_ID: OnceLock<String> = OnceLock::new();

pub fn get_domain_reach_id() -> &'static str {
    DOMAIN_REACH_ID.get().expect("get domain id before it's initialized")
}

pub fn get_tardis_inst() -> TardisFunsInst {
    TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None)
}

pub fn get_sms_client() -> Arc<SmsClient> {
    static SMS_CLIENT: OnceLock<Arc<SmsClient>> = OnceLock::new();
    SMS_CLIENT
        .get_or_init(|| {
            // this would block thread but it's ok
            let config = TardisFuns::cs_config::<ReachConfig>(DOMAIN_CODE);
            let sms_config = &config.sms;
            let base_url = sms_config.base_url.parse().expect("invalid sms base url");
            let callback_url = sms_config.status_call_back.as_ref().map(|x| x.parse().expect("invalid sms status_call_back url"));
            SmsClient::new(base_url, &sms_config.app_key, &sms_config.app_secret, callback_url).into()
        })
        .clone()
}

pub fn get_mail_client() -> MailClient {
    // it's cheap, `MailClient` inner is a single static ref
    MailClient::new()
}
