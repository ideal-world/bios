use crate::consts::get_tardis_inst;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct MailClient {
    pub(super) inner: &'static tardis::mail::mail_client::TardisMailClient,
}

impl MailClient {
    pub fn new() -> Self {
        Self { inner: get_tardis_inst().mail() }
    }
}

impl Default for MailClient {
    fn default() -> Self {
        Self::new()
    }
}
