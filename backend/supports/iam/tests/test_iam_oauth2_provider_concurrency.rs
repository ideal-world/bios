use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bios_basic::test::init_test_container;
use bios_iam::basic::domain::iam_oauth2_provider_grant;
use bios_iam::basic::dto::iam_cert_conf_dto::IamCertConfOAuth2AddOrModifyReq;
use bios_iam::basic::serv::iam_cert_oauth2_serv::{IamCertOAuth2Serv, IamCertOAuth2TokenInfo};
use bios_iam::iam_constants;
use bios_iam::iam_enumeration::IamCertOAuth2Supplier;
use tardis::basic::{dto::TardisContext, field::TrimString, result::TardisResult};
use tardis::chrono::Utc;
use tardis::crypto::crypto_aead::algorithm::Aes256Gcm;
use tardis::db::sea_orm::{EntityTrait, Set};
use tardis::tokio;
use tardis::web::poem::{endpoint::make, http::StatusCode, listener::TcpAcceptor, Body, Request, Response, Route, Server};
use tardis::{TardisFuns, TardisFunsInst};

fn lease_key(account: &str) -> String {
    format!("iam:oauth2:provider-refresh:BiosIam:{account}")
}

async fn seed_token(account: &str, conf_id: &str, expired: bool, funs: &TardisFunsInst) -> TardisResult<()> {
    let token = IamCertOAuth2TokenInfo {
        open_id: format!("subject-{account}"),
        access_token: format!("cached-{account}"),
        refresh_token: Some(format!("refresh-{account}")),
        token_expires_ms: Some(3_600_000),
        expires_at_ms: Some(Utc::now().timestamp_millis() + if expired { -1_000 } else { 3_600_000 }),
        scope: Some("iam.app.read".to_string()),
        provider_cert_conf_id: Some(conf_id.to_string()),
        union_id: None,
    };
    let serialized = TardisFuns::json.obj_to_string(&token)?;
    let key = format!("{}BiosIam:{account}", funs.conf::<bios_iam::iam_config::IamConfig>().cache_key_oauth2_provider_token_);
    funs.cache().set_ex(&key, &serialized, 600).await?;
    let id = TardisFuns::crypto.digest.sha256(TardisFuns::json.obj_to_string(&(account, conf_id, &token.open_id))?)?;
    let nonce = TardisFuns::crypto.aead.random_nonce::<Aes256Gcm>();
    let (ciphertext, _) = TardisFuns::crypto.aead.encrypt::<Aes256Gcm>(&[7; 32], id.as_bytes(), &nonce, serialized.as_bytes())?;
    iam_oauth2_provider_grant::Entity::insert(iam_oauth2_provider_grant::ActiveModel {
        id: Set(id),
        account_id: Set(account.to_string()),
        cert_conf_id: Set(conf_id.to_string()),
        external_subject: Set(token.open_id),
        encrypted_token: Set(format!("{}:{}", TardisFuns::crypto.hex.encode(nonce), TardisFuns::crypto.hex.encode(ciphertext))),
        update_time: Set(Utc::now().to_rfc3339()),
    })
    .exec(funs.db().raw_conn())
    .await?;
    Ok(())
}

#[tokio::test]
async fn bios_provider_contention_waits_and_reuses_the_committed_refresh() -> TardisResult<()> {
    std::env::set_var("TARDIS_CSM.IAM.OAUTH2_PROVIDER_GRANT_KEY", "07".repeat(32));
    let _containers = init_test_container::init(None).await?;
    bios_iam::iam_initializer::init_db(iam_constants::get_tardis_inst()).await?;
    let funs = iam_constants::get_tardis_inst();

    let refreshes = Arc::new(AtomicUsize::new(0));
    let delay_ms = Arc::new(AtomicUsize::new(6_000));
    let status = Arc::new(AtomicUsize::new(200));
    let (count, delay, provider_status) = (refreshes.clone(), delay_ms.clone(), status.clone());
    let endpoint = make(move |request: Request| {
        let (count, delay, status) = (count.clone(), delay.clone(), provider_status.clone());
        async move {
            let body = request.into_body().into_string().await.unwrap();
            assert!(body.contains("refresh_token"));
            count.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(delay.load(Ordering::SeqCst) as u64)).await;
            if status.load(Ordering::SeqCst) == 503 {
                return Response::builder()
                    .status(StatusCode::SERVICE_UNAVAILABLE)
                    .content_type("application/json")
                    .body(Body::from_string(r#"{"code":"503","msg":"provider unavailable"}"#.to_string()));
            }
            Response::builder().status(StatusCode::OK).content_type("application/json").body(Body::from_string(
                r#"{"code":"200","msg":"","data":{"access_token":"fresh-access","token_type":"Bearer","expires_in":3600,"refresh_token":"rotated-refresh","scope":"iam.app.read"}}"#.to_string(),
            ))
        }
    });
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let address = listener.local_addr().unwrap();
    let acceptor = TcpAcceptor::from_tokio(listener).unwrap();
    let server = tokio::spawn(async move { Server::new_with_acceptor(acceptor).run(Route::new().at("/cp/oauth2/token", endpoint)).await.unwrap() });
    let conf_id = IamCertOAuth2Serv::add_cert_conf(
        IamCertOAuth2Supplier::BiosIam,
        &IamCertConfOAuth2AddOrModifyReq {
            supplier: TrimString("BiosIam".to_string()),
            ak: TrimString("test-client".to_string()),
            sk: TrimString("test-secret".to_string()),
            base_url: Some(format!("http://{address}")),
        },
        "",
        &funs,
        &TardisContext::default(),
    )
    .await?;
    seed_token("valid", &conf_id, false, &funs).await?;
    seed_token("expired", &conf_id, true, &funs).await?;
    seed_token("other", &conf_id, false, &funs).await?;
    seed_token("outage", &conf_id, true, &funs).await?;

    // 有效凭据争锁时在 BIOS 内等待，其他账号不应一起阻塞。
    funs.cache().set_ex(&lease_key("valid"), "test-holder", 120).await?;
    let other = tokio::time::timeout(
        Duration::from_secs(2),
        IamCertOAuth2Serv::get_provider_token(IamCertOAuth2Supplier::BiosIam, "other", "tenant/app", &funs),
    )
    .await
    .unwrap()?;
    assert_eq!(other.access_token, "cached-other");
    let (first, second, released) = tokio::join!(
        IamCertOAuth2Serv::get_provider_token(IamCertOAuth2Supplier::BiosIam, "valid", "tenant/app", &funs),
        IamCertOAuth2Serv::get_provider_token(IamCertOAuth2Supplier::BiosIam, "valid", "tenant/app", &funs),
        async {
            tokio::time::sleep(Duration::from_millis(200)).await;
            funs.cache().del(&lease_key("valid")).await
        },
    );
    released?;
    assert_eq!(first?.access_token, "cached-valid");
    assert_eq!(second?.access_token, "cached-valid");
    assert_eq!(refreshes.load(Ordering::SeqCst), 0);

    // 刷新超过旧的五秒等待窗口后，所有读取者应复用同一次刷新结果。
    let (first, second) = tokio::time::timeout(Duration::from_secs(20), async {
        tokio::join!(
            IamCertOAuth2Serv::get_provider_token(IamCertOAuth2Supplier::BiosIam, "expired", "tenant/app", &funs),
            IamCertOAuth2Serv::get_provider_token(IamCertOAuth2Supplier::BiosIam, "expired", "tenant/app", &funs),
        )
    })
    .await
    .unwrap();
    assert_eq!(first?.access_token, "fresh-access");
    assert_eq!(second?.refresh_token.as_deref(), Some("rotated-refresh"));
    assert_eq!(refreshes.load(Ordering::SeqCst), 1);

    // 两个显式刷新也应合并；之后独立发起的新刷新仍需执行。
    let (first, second) = tokio::time::timeout(Duration::from_secs(20), async {
        tokio::join!(
            IamCertOAuth2Serv::refresh_provider_token(IamCertOAuth2Supplier::BiosIam, "valid", "tenant/app", &funs),
            IamCertOAuth2Serv::refresh_provider_token(IamCertOAuth2Supplier::BiosIam, "valid", "tenant/app", &funs),
        )
    })
    .await
    .unwrap();
    assert_eq!(first?.access_token, "fresh-access");
    assert_eq!(second?.access_token, "fresh-access");
    assert_eq!(refreshes.load(Ordering::SeqCst), 2);
    delay_ms.store(0, Ordering::SeqCst);
    // 缓存落后于持久化凭据时，新请求仍须执行显式刷新，不能误判为刚完成的并发刷新。
    let cache_key = format!("{}BiosIam:valid", funs.conf::<bios_iam::iam_config::IamConfig>().cache_key_oauth2_provider_token_);
    let mut stale: IamCertOAuth2TokenInfo = TardisFuns::json.str_to_obj(&funs.cache().get(&cache_key).await?.unwrap())?;
    stale.access_token = "stale-cache-access".to_string();
    funs.cache().set_ex(&cache_key, &TardisFuns::json.obj_to_string(&stale)?, 600).await?;
    IamCertOAuth2Serv::refresh_provider_token(IamCertOAuth2Supplier::BiosIam, "valid", "tenant/app", &funs).await?;
    assert_eq!(refreshes.load(Ordering::SeqCst), 3);

    // Provider 故障要正常返回并释放租约，恢复后应能重新执行。
    status.store(503, Ordering::SeqCst);
    let error = IamCertOAuth2Serv::get_provider_token(IamCertOAuth2Supplier::BiosIam, "outage", "tenant/app", &funs).await.unwrap_err();
    assert!(error.code.starts_with("502"), "Provider outage must retain the existing gateway error mapping");
    assert_eq!(refreshes.load(Ordering::SeqCst), 4);
    assert!(!funs.cache().exists(&lease_key("outage")).await?);
    status.store(200, Ordering::SeqCst);
    assert_eq!(
        IamCertOAuth2Serv::get_provider_token(IamCertOAuth2Supplier::BiosIam, "outage", "tenant/app", &funs).await?.access_token,
        "fresh-access"
    );
    assert_eq!(refreshes.load(Ordering::SeqCst), 5);
    server.abort();
    Ok(())
}
