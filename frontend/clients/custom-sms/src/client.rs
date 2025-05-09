use std::sync::Arc;

use tardis::{
    basic::result::TardisResult,
    url::Url,
    web::reqwest::{
        header::{HeaderMap, HeaderValue},
        Client as ReqwestClient,
    },
};

use crate::{
    consts::*,
    model::{SendCommonMessageRequest, SendCommonMessageResponse},
    CustomSmsResponse,
};

#[derive(Debug, Clone)]
pub struct Client {
    inner: Arc<ReqwestClient>,
    base_url: Url,
}

impl Client {
    pub fn init(url: Url, app_id: HeaderValue, app_pwd: HeaderValue) -> TardisResult<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(APP_AUTH_HEADER_KEY, app_pwd);
        headers.insert(APP_ID_HEADER_KEY, app_id);
        let client = Client {
            inner: Arc::new(ReqwestClient::builder().default_headers(headers).build()?),
            base_url: url,
        };
        Ok(client)
    }
    pub fn get_base_url(&self) -> Url {
        self.base_url.clone()
    }
    pub fn get_url(&self, path: &str) -> Url {
        let mut url = self.base_url.clone();
        url.set_path(path);
        url
    }
}

// methods
impl Client {
    pub async fn send_message(
        &self,
        req: &SendCommonMessageRequest,
    ) -> TardisResult<CustomSmsResponse<SendCommonMessageResponse>> {
        let url = self.get_url("ums-rest-api/msgRest/sendCommonMsgJson");
        let resp = self
            .inner
            .post(url)
            .json(&req)
            .send()
            .await?
            .json::<CustomSmsResponse<SendCommonMessageResponse>>()
            .await?;
        Ok(resp)
    }
    pub async fn batch_send_messages(
        &self,
        req: &[&SendCommonMessageRequest],
    ) -> TardisResult<CustomSmsResponse<Vec<SendCommonMessageResponse>>> {
        let url = self.get_url("ums-rest-api/msgRest/sendCommonMsgListJson");
        let resp = self
            .inner
            .post(url)
            .json(&req)
            .send()
            .await?
            .json::<CustomSmsResponse<Vec<SendCommonMessageResponse>>>()
            .await?;
        Ok(resp)
    }
}
