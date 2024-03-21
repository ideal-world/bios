use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumKindFilterReq};
use bios_basic::rbum::dto::rbum_item_dto::RbumItemAddReq;
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemServ;
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindServ;
use bios_basic::{rbum::rbum_config::RbumConfig, test::test_http_client::TestHttpClient};
use bios_client_hwsms::{SmsId, SmsResponse};
use bios_reach::reach_consts::{get_tardis_inst, DOMAIN_CODE, IAM_KEY_PHONE_V_CODE, RBUM_KIND_CODE_REACH_MESSAGE, REACH_INIT_OWNER};
use bios_reach::reach_send_channel::SendChannelMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tardis::testcontainers::GenericImage;
use tardis::tokio::sync::RwLock;
use tardis::web::poem_openapi::param::Path;
use tardis::web::poem_openapi::payload::{Form, Json};
use tardis::web::poem_openapi::{self, Object};
use tardis::web::web_resp::{TardisApiResult, TardisResp};
use tardis::web::web_server::{WebServerModule, WebServerModuleOption};
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    test::test_container::TardisTestContainer,
    testcontainers::{clients::Cli, Container},
    TardisFuns,
};
use tardis::{log, rand, serde_json};
use testcontainers_modules::redis::Redis;
pub struct Holder<'d> {
    pub db: Container<'d, GenericImage>,
    pub cache: Container<'d, Redis>,
    pub mq: Container<'d, GenericImage>,
    pub sms_mocker: HwSmsMockerApi,
    pub iam_mocker: IamMockerApi,
}

pub const TEST_OWNER: &str = "test-reach";
pub fn get_test_ctx() -> &'static TardisContext {
    static TEST_CTX: OnceLock<TardisContext> = OnceLock::new();
    TEST_CTX.get_or_init(|| TardisContext {
        owner: TEST_OWNER.to_string(),
        ..Default::default()
    })
}

#[allow(dead_code)]
pub async fn init_tardis(docker: &Cli) -> TardisResult<Holder> {
    let reldb_container = TardisTestContainer::postgres_custom(None, docker);
    let port = reldb_container.get_host_port_ipv4(5432);
    let url = format!("postgres://postgres:123456@127.0.0.1:{port}/test");
    std::env::set_var("TARDIS_FW.DB.URL", url);
    let redis_container = TardisTestContainer::redis_custom(docker);
    let port = redis_container.get_host_port_ipv4(6379);
    let url = format!("redis://127.0.0.1:{port}/0");
    std::env::set_var("TARDIS_FW.CACHE.URL", url);
    let rabbit_container = TardisTestContainer::rabbit_custom(docker);
    let port = rabbit_container.get_host_port_ipv4(5672);
    let url = format!("amqp://guest:guest@127.0.0.1:{port}/%2f");
    std::env::set_var("TARDIS_FW.MQ.URL", url);

    TardisFuns::init(Some("tests/config")).await?;
    bios_basic::rbum::rbum_initializer::init(DOMAIN_CODE, RbumConfig::default()).await?;
    bios_basic::rbum::rbum_initializer::init("", RbumConfig::default()).await?;
    let web_server = TardisFuns::web_server();
    bios_reach::init(
        &web_server,
        SendChannelMap::new().with_arc_channel(bios_client_hwsms::SmsClient::from_reach_config()).with_arc_channel(get_tardis_inst().mail()),
    )
    .await?;
    let sms_mocker = HwSmsMockerApi::default();
    let iam_mocker = IamMockerApi::default();
    web_server.add_module("sms", WebServerModule::from(sms_mocker.clone()).options(WebServerModuleOption { uniform_error: false })).await;
    web_server.add_module("iam", iam_mocker.clone()).await;
    let funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    let ctx = TardisContext {
        owner: REACH_INIT_OWNER.into(),
        ..Default::default()
    };
    let rel_rbum_kind_id = RbumKindServ::find_one_rbum(
        &RbumKindFilterReq {
            basic: RbumBasicFilterReq {
                code: Some(RBUM_KIND_CODE_REACH_MESSAGE.into()),
                ..Default::default()
            },
            ..Default::default()
        },
        &funs,
        &ctx,
    )
    .await?
    .expect("fail to find kind")
    .id;
    let rel_rbum_domain_id = RbumDomainServ::find_one_rbum(
        &RbumBasicFilterReq {
            code: Some(DOMAIN_CODE.into()),
            ..Default::default()
        },
        &funs,
        &ctx,
    )
    .await?
    .expect("fail to find domain")
    .id;
    RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            code: Some("reach-test".into()),
            name: "reach-test".into(),
            scope_level: Some(RbumScopeLevelKind::Root),
            id: Some(TEST_OWNER.into()),
            rel_rbum_kind_id,
            rel_rbum_domain_id,
            disabled: None,
        },
        &funs,
        &ctx,
    )
    .await?;
    web_server.start().await?;
    let holder = Holder {
        db: reldb_container,
        cache: redis_container,
        mq: rabbit_container,
        sms_mocker,
        iam_mocker,
    };
    Ok(holder)
}

#[allow(dead_code)]
pub async fn start_web_server() -> TardisResult<()> {
    TardisFuns::web_server().start().await
}

#[allow(dead_code)]
pub fn get_client(url: &str, ctx: &TardisContext) -> TestHttpClient {
    let mut client: TestHttpClient = TestHttpClient::new(url.into());
    client.set_auth(ctx).unwrap();
    client
}

#[allow(dead_code)]
pub fn wait_for_press() {
    use std::io::*;
    println!("Press any key to continue");
    stdin().read_line(&mut String::new()).expect("fail to read from stdin");
}

#[allow(dead_code)]
pub fn random_string(size: usize) -> String {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    thread_rng().sample_iter(&Alphanumeric).take(size).map(|x| x as char).collect()
}

#[allow(dead_code)]
#[derive(Clone, Default)]
pub struct HwSmsMockerApi {
    pub sent_messages: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

#[allow(dead_code)]
impl HwSmsMockerApi {
    pub async fn get_latest_message(&self, user: &str) -> Option<String> {
        self.sent_messages.read().await.get(user).and_then(|x| x.last().cloned())
    }
}

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct SendSmsRequest {
    pub from: String,
    pub status_callback: Option<String>,
    pub extend: Option<String>,
    pub to: String,
    pub template_id: String,
    pub template_paras: String,
    pub signature: Option<String>,
}

#[poem_openapi::OpenApi]
impl HwSmsMockerApi {
    #[oai(path = "/batchSendSms/v1", method = "post")]
    async fn get_ct_account(&self, request: Form<SendSmsRequest>) -> Json<serde_json::Value> {
        log::debug!("revieved sms request: {:?}", request);
        for to in request.to.split(',') {
            self.sent_messages.write().await.entry(to.into()).or_insert(vec![]).push(request.template_paras.clone());
        }
        let response = SmsResponse {
            code: "200".into(),
            description: "OK".into(),
            result: Some(
                request
                    .to
                    .split(',')
                    .map(|x| SmsId {
                        from: request.from.clone(),
                        sms_msg_id: random_string(16),
                        origin_to: x.into(),
                        status: "OK".into(),
                        create_time: tardis::chrono::Utc::now().to_rfc3339(),
                    })
                    .collect::<Vec<_>>(),
            ),
        };
        Json(serde_json::to_value(response).unwrap())
    }
}

#[derive(Default, Clone)]
pub struct IamMockerApi {}
#[derive(Debug, Serialize, Deserialize, Object)]
pub struct GetAccountResp {
    pub roles: HashMap<String, String>,
    pub certs: HashMap<String, String>,
    pub orgs: Vec<String>,
}

#[poem_openapi::OpenApi]
impl IamMockerApi {
    #[oai(path = "/ct/account/:id", method = "get")]
    async fn get_ct_account(&self, id: Path<String>) -> TardisApiResult<GetAccountResp> {
        log::info!("revieved iam request: {:?}", &id.0);
        let mut resp = GetAccountResp {
            roles: HashMap::new(),
            certs: HashMap::new(),
            orgs: vec![],
        };
        resp.certs.insert(IAM_KEY_PHONE_V_CODE.into(), id.0);
        TardisResp::ok(resp)
    }
}
