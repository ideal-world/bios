use serde::{Deserialize, Serialize};
use tardis::{serde_json::Value, web::poem_openapi};

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ReldbTxResp {
    pub tx_id: String,
    pub exp_ts_at: i64,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ReldbDdlReq {
    #[oai(validator(min_length = "2"))]
    pub sql: String,
    pub params: Value,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ReldbDmlReq {
    #[oai(validator(min_length = "2"))]
    pub sql: String,
    pub params: Value,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ReldbDmlResp {
    pub affected_rows: u64,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct ReldbDqlReq {
    #[oai(validator(min_length = "2"))]
    pub sql: String,
    pub params: Value,
}

// #[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
// pub struct ReldbUpsertReq {
//     #[oai(validator(min_length = "2"))]
//     pub table_name: String,
//     #[oai(validator(min_length = "2"))]
//     pub pk_name: String,
//     #[oai(validator(min_length = "2"))]
//     pub records: JsonArray,
// }

// #[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
// pub struct ReldbDeleteReq {
//     #[oai(validator(min_length = "2"))]
//     pub table_name: String,
//     pub pk_ids: Vec<String>,
// }
