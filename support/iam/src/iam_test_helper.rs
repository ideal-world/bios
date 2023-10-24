use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::config::config_dto::WebClientModuleConfig;
use tardis::log::{info, warn};
use tardis::web::poem_openapi::types::{ParseFromJSON, ToJSON};
use tardis::web::web_client::TardisWebClient;
use tardis::web::web_resp::{TardisResp, Void};
use tardis::TardisFuns;

use crate::basic::dto::iam_account_dto::IamAccountInfoResp;
use crate::basic::dto::iam_cert_dto::IamContextFetchReq;
use crate::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;

pub struct BIOSWebTestClient {
    client: TardisWebClient,
    context: TardisContext,
    base_url: String,
}

impl BIOSWebTestClient {
    pub fn new(base_url: String) -> BIOSWebTestClient {
        BIOSWebTestClient {
            client: TardisWebClient::init(&WebClientModuleConfig::builder().connect_timeout_sec(600u64).build()).unwrap(),
            context: Default::default(),
            base_url,
        }
    }

    pub async fn set_auth(&mut self, token: &str, app_id: Option<String>) -> TardisResult<()> {
        let context: String = self.put("/cp/context", &IamContextFetchReq { token: token.to_string(), app_id }).await;
        self.context = TardisFuns::json.str_to_obj(&TardisFuns::crypto.base64.decode_to_string(&context)?)?;
        let fw_config = TardisFuns::fw_config();
        let web_server_config = fw_config.web_server();
        let context_header_name = web_server_config.context_conf.context_header_name.as_str();
        self.set_default_header(context_header_name, &context);
        Ok(())
    }

    pub fn context(&self) -> &TardisContext {
        &self.context
    }

    pub fn set_default_header(&mut self, key: &str, value: &str) {
        self.client.remove_default_header(key);
        self.client.set_default_header(key, value);
    }

    pub async fn login(
        &mut self,
        user_name: &str,
        password: &str,
        tenant_id: Option<String>,
        app_id: Option<String>,
        flag: Option<String>,
        set_auth: bool,
    ) -> TardisResult<IamAccountInfoResp> {
        // Login
        let account: IamAccountInfoResp = self
            .put(
                "/cp/login/userpwd",
                &IamCpUserPwdLoginReq {
                    ak: TrimString(user_name.to_string()),
                    sk: TrimString(password.to_string()),
                    tenant_id,
                    flag,
                },
            )
            .await;
        // Find Context
        if set_auth {
            self.set_auth(&account.token, app_id).await?;
        }
        Ok(account)
    }

    pub async fn get_to_str(&self, url: &str) -> String {
        self.client.get_to_str(format!("{}{}", self.base_url, url).as_str(), None).await.unwrap().body.unwrap()
    }

    pub async fn get<T>(&self, url: &str) -> T
    where
        T: DeserializeOwned + ParseFromJSON + ToJSON + Serialize + Send + Sync + Debug,
    {
        let result: TardisResp<T> = self.client.get::<TardisResp<T>>(format!("{}{}", self.base_url, url).as_str(), None).await.unwrap().body.unwrap();
        if result.code != "200000000000" {
            warn!("========[{}]|{}", result.code, result.msg);
        }
        info!("<<<<[GET]|{}:{:#?}", url, result);
        result.data.unwrap()
    }

    pub async fn get_resp<T>(&self, url: &str) -> TardisResp<T>
    where
        T: DeserializeOwned + ParseFromJSON + ToJSON + Serialize + Send + Sync + Debug,
    {
        let result: TardisResp<T> = self.client.get::<TardisResp<T>>(format!("{}{}", self.base_url, url).as_str(), None).await.unwrap().body.unwrap();
        if result.code != "200000000000" {
            warn!("========[{}]|{}", result.code, result.msg);
        }
        info!("<<<<[GET]|{}:{:#?}", url, result);
        result
    }

    pub async fn delete(&self, url: &str) {
        let result: TardisResp<Void> = self.client.delete(format!("{}{}", self.base_url, url).as_str(), None).await.unwrap().body.unwrap();
        if result.code != "200000000000" {
            warn!("========[{}]|{}", result.code, result.msg);
        }
        info!("<<<<[DELETE]|{}:{:#?}", url, result);
    }

    pub async fn delete_resp(&self, url: &str) -> TardisResp<Void> {
        let result: TardisResp<Void> = self.client.delete(format!("{}{}", self.base_url, url).as_str(), None).await.unwrap().body.unwrap();
        if result.code != "200000000000" {
            warn!("========[{}]|{}", result.code, result.msg);
        }
        info!("<<<<[DELETE]|{}:{:#?}", url, result);
        result
    }

    pub async fn post<B: Serialize + Debug, T>(&self, url: &str, body: &B) -> T
    where
        T: DeserializeOwned + ParseFromJSON + ToJSON + Serialize + Send + Sync + Debug,
    {
        info!(">>>>[POST]|{}:{:#?}", url, body);
        let result: TardisResp<T> = self.client.post::<B, TardisResp<T>>(format!("{}{}", self.base_url, url).as_str(), body, None).await.unwrap().body.unwrap();
        if result.code != "200000000000" {
            warn!("========[{}]|{}", result.code, result.msg);
        }
        info!("<<<<[POST]|{}:{:#?}", url, result);
        result.data.unwrap()
    }

    pub async fn post_resp<B: Serialize + Debug, T>(&self, url: &str, body: &B) -> TardisResp<T>
    where
        T: DeserializeOwned + ParseFromJSON + ToJSON + Serialize + Send + Sync + Debug,
    {
        info!(">>>>[POST]|{}:{:#?}", url, body);
        let result: TardisResp<T> = self.client.post::<B, TardisResp<T>>(format!("{}{}", self.base_url, url).as_str(), body, None).await.unwrap().body.unwrap();
        if result.code != "200000000000" {
            warn!("========[{}]|{}", result.code, result.msg);
        }
        info!("<<<<[POST]|{}:{:#?}", url, result);
        result
    }

    pub async fn put<B: Serialize + Debug, T>(&self, url: &str, body: &B) -> T
    where
        T: DeserializeOwned + ParseFromJSON + ToJSON + Serialize + Send + Sync + Debug,
    {
        info!(">>>>[PUT]|{}:{:#?}", url, body);
        let result: TardisResp<T> = self.client.put::<B, TardisResp<T>>(format!("{}{}", self.base_url, url).as_str(), body, None).await.unwrap().body.unwrap();
        if result.code != "200000000000" {
            warn!("========[{}]|{}", result.code, result.msg);
        }
        info!("<<<<[PUT]|{}:{:#?}", url, result);
        result.data.unwrap()
    }

    pub async fn put_resp<B: Serialize + Debug, T>(&self, url: &str, body: &B) -> TardisResp<T>
    where
        T: DeserializeOwned + ParseFromJSON + ToJSON + Serialize + Send + Sync + Debug,
    {
        info!(">>>>[PUT]|{}:{:#?}", url, body);
        let result: TardisResp<T> = self.client.put::<B, TardisResp<T>>(format!("{}{}", self.base_url, url).as_str(), body, None).await.unwrap().body.unwrap();
        if result.code != "200000000000" {
            warn!("========[{}]|{}", result.code, result.msg);
        }
        info!("<<<<[PUT]|{}:{:#?}", url, result);
        result
    }

    pub async fn patch<B: Serialize + Debug, T>(&self, url: &str, body: &B) -> T
    where
        T: DeserializeOwned + ParseFromJSON + ToJSON + Serialize + Send + Sync + Debug,
    {
        info!(">>>>[PATCH]|{}:{:#?}", url, body);
        let result: TardisResp<T> = self.client.patch::<B, TardisResp<T>>(format!("{}{}", self.base_url, url).as_str(), body, None).await.unwrap().body.unwrap();
        if result.code != "200000000000" {
            warn!("========[{}]|{}", result.code, result.msg);
        }
        info!("<<<<[PATCH]|{}:{:#?}", url, result);
        result.data.unwrap()
    }

    pub async fn patch_resp<B: Serialize + Debug, T>(&self, url: &str, body: &B) -> TardisResp<T>
    where
        T: DeserializeOwned + ParseFromJSON + ToJSON + Serialize + Send + Sync + Debug,
    {
        info!(">>>>[PATCH]|{}:{:#?}", url, body);
        let result: TardisResp<T> = self.client.patch::<B, TardisResp<T>>(format!("{}{}", self.base_url, url).as_str(), body, None).await.unwrap().body.unwrap();
        if result.code != "200000000000" {
            warn!("========[{}]|{}", result.code, result.msg);
        }
        info!("<<<<[PATCH]|{}:{:#?}", url, result);
        result
    }
}
