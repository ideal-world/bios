use poem_openapi::types::{ParseFromJSON, ToJSON};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tardis::web::web_client::TardisWebClient;
use tardis::web::web_resp::TardisResp;

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

    pub fn set_default_header(&mut self, key: &str, value: &str) {
        self.client.set_default_header(key, value);
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
