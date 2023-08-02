use bios_sdk_invoke::clients::SimpleInvokeClient;
use tardis::basic::dto::TardisContext;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::{Json, Path, Query};

use tardis::web::poem_openapi;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp};


#[derive(Clone, Default)]
/// 用户触达消息-公共控制台
pub struct Api;

pub struct Client {

}

impl SimpleInvokeClient for Client {
    const DOMAIN_CODE: &'static str = "test";
    fn get_ctx(&self) -> &'_ TardisContext {
        unimplemented!()
    }

    fn get_base_url(&self) -> &str {
        unimplemented!()
    }

    fn get_funs(&self) -> &tardis::TardisFunsInst {
        todo!()
    }
}
use simple_invoke_client_macro::simple_invoke_client;
#[simple_invoke_client(Client)]
#[poem_openapi::OpenApi(prefix_path = "/ct/msg")]
impl Api {
    /// 获取所有用户触达消息数据分页
    #[oai(method = "get", path = "/page")]
    pub async fn get_page(
        &self,
        page_number: Query<Option<u32>>,
        page_size: Query<Option<u32>>,
        TardisContextExtractor(ctx): TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<String>> {
        TardisResp::ok(TardisPage {
            page_number: 1,
            page_size: 10,
            total_size: 1,
            records: vec!["hello".to_string()],
        })
    }
        /// 获取所有用户触达消息数据分页
    #[oai(method = "get", path = "/page/:page_number/size/:page_size")]
    pub async fn get_page_path(
        &self,
        page_number: Path<u32>,
        page_size: Path<u32>,
        TardisContextExtractor(ctx): TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<String>> {
        TardisResp::ok(TardisPage {
            page_number: 1,
            page_size: 10,
            total_size: 1,
            records: vec!["hello".to_string()],
        })
    }
}

async fn test() {
    let client = Client {

    };
    let resp = client.get_page(None, None).await;
    let resp = client.get_page_path(1, 2).await;
}