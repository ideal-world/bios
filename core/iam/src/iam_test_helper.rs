use poem_openapi::types::{ParseFromJSON, ToJSON};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::web::web_client::TardisWebClient;
use tardis::web::web_resp::TardisResp;
use tardis::TardisFuns;

use crate::basic::dto::iam_cert_dto::IamContextFetchReq;

pub struct BIOSWebTestClient {
    client: TardisWebClient,
    base_url: String,
}

impl BIOSWebTestClient {
    pub fn new(base_url: String) -> BIOSWebTestClient {
        BIOSWebTestClient {
            client: TardisWebClient::init(600).unwrap(),
            base_url,
        }
    }

    pub async fn set_auth(&mut self, token: &str, app_id: Option<String>) -> TardisResult<()> {
        let context: TardisContext = self.put("/cp/context", &IamContextFetchReq { token: token.to_string(), app_id }).await;
        self.set_default_header(
            &TardisFuns::fw_config().web_server.context_conf.context_header_name,
            TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&context)?).as_str(),
        );
        Ok(())
    }

    pub fn set_default_header(&mut self, key: &str, value: &str) {
        self.client.set_default_header(key, value);
    }

    pub async fn get_to_str(&self, url: &str) -> String {
        self.client.get_to_str(format!("{}{}", self.base_url, url).as_str(), None).await.unwrap().body.unwrap()
    }

    pub async fn get<T>(&self, url: &str) -> T
    where
        T: DeserializeOwned + ParseFromJSON + ToJSON + Serialize + Send + Sync,
    {
        let result: TardisResp<T> = self.client.get::<TardisResp<T>>(format!("{}{}", self.base_url, url).as_str(), None).await.unwrap().body.unwrap();
        result.data.unwrap()
    }

    pub async fn delete(&self, url: &str) {
        self.client.delete(format!("{}{}", self.base_url, url).as_str(), None).await.unwrap();
    }

    pub async fn post<B: Serialize, T>(&self, url: &str, body: &B) -> T
    where
        T: DeserializeOwned + ParseFromJSON + ToJSON + Serialize + Send + Sync,
    {
        let result: TardisResp<T> = self.client.post::<B, TardisResp<T>>(format!("{}{}", self.base_url, url).as_str(), body, None).await.unwrap().body.unwrap();
        result.data.unwrap()
    }

    pub async fn put<B: Serialize, T>(&self, url: &str, body: &B) -> T
    where
        T: DeserializeOwned + ParseFromJSON + ToJSON + Serialize + Send + Sync,
    {
        let result: TardisResp<T> = self.client.put::<B, TardisResp<T>>(format!("{}{}", self.base_url, url).as_str(), body, None).await.unwrap().body.unwrap();
        result.data.unwrap()
    }

    pub async fn patch<B: Serialize, T>(&self, url: &str, body: &B) -> T
    where
        T: DeserializeOwned + ParseFromJSON + ToJSON + Serialize + Send + Sync,
    {
        let result: TardisResp<T> = self.client.patch::<B, TardisResp<T>>(format!("{}{}", self.base_url, url).as_str(), body, None).await.unwrap().body.unwrap();
        result.data.unwrap()
    }
}
