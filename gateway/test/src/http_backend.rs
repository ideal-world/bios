use serde::{Deserialize, Serialize};

use tardis::basic::field::TrimString;
use tardis::log;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Path;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::TardisApiResult;
use tardis::web::web_resp::TardisResp;

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct TestDetailResp {
    pub id: i32,
    pub code: String,
    pub description: String,
    pub done: bool,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct TestAddReq {
    pub code: TrimString,
    pub description: String,
    pub done: bool,
}

pub struct TestApi;

#[poem_openapi::OpenApi(prefix_path = "/echo")]
impl TestApi {
    #[oai(path = "/:id", method = "post")]
    async fn echo(&self, id: Path<i32>, add_req: Json<TestAddReq>) -> TardisApiResult<TestDetailResp> {
        log::info!("-------echo----------");
        TardisResp::ok(TestDetailResp {
            id: id.0,
            code: add_req.0.code.to_string(),
            description: add_req.0.description,
            done: add_req.0.done,
        })
    }
}
