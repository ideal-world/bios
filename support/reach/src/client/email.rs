use tardis::TardisFuns;

#[derive(Clone)]
pub struct MailClient {
    pub(super) inner: &'static tardis::mail::mail_client::TardisMailClient,
}

impl MailClient {
    pub fn new() -> Self {
        Self { inner: TardisFuns::mail() }
    }
}

impl Default for MailClient {
    fn default() -> Self {
        Self::new()
    }
}
