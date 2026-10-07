use std::sync::{Arc, Mutex};
use std::time::Duration;

use bios_basic::test::init_test_container;
use bios_iam::basic::domain::iam_oauth2_provider_grant;
use bios_iam::basic::dto::iam_cert_conf_dto::IamCertConfOAuth2AddOrModifyReq;
use bios_iam::basic::serv::iam_cert_oauth2_serv::{IamCertOAuth2Serv, IamCertOAuth2TokenInfo};
use bios_iam::basic::serv::iam_cert_serv::IamCertServ;
use bios_iam::iam_constants;
use bios_iam::iam_enumeration::{IamCertExtKind, IamCertOAuth2Supplier};
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::chrono::Utc;
use tardis::crypto::crypto_aead::algorithm::Aes256Gcm;
use tardis::db::sea_orm::{EntityTrait, Set};
use tardis::tokio;
use tardis::web::poem::{endpoint::make, http::StatusCode, listener::TcpAcceptor, Body, Request, Response, Route, Server};
use tardis::TardisFuns;

const ACCOUNT_ID: &str = "oauth2-refresh-account";

async fn set_provider_cache_oom(funs: &tardis::TardisFunsInst, enabled: bool) -> TardisResult<()> {
    let mut connection = funs.cache().cmd().await?;
    let _: String = tardis::cache::cmd("CONFIG").arg("SET").arg("maxmemory-policy").arg("noeviction").query_async(&mut connection).await?;
    let maxmemory = if enabled { "1" } else { "0" };
    let _: String = tardis::cache::cmd("CONFIG").arg("SET").arg("maxmemory").arg(maxmemory).query_async(&mut connection).await?;
    Ok(())
}

#[tokio::test]
async fn get_provider_token_refreshes_expired_bios_grant_through_mock_provider() -> TardisResult<()> {
    std::env::set_var("TARDIS_CSM.IAM.OAUTH2_PROVIDER_GRANT_KEY", "07".repeat(32));
    let _containers = init_test_container::init(None).await?;
    let _ = bios_iam::iam_initializer::init_db(iam_constants::get_tardis_inst()).await?;
    let funs = iam_constants::get_tardis_inst();
    let request_bodies = Arc::new(Mutex::new(Vec::new()));
    let request_bodies_for_endpoint = request_bodies.clone();
    let endpoint_funs = Arc::new(iam_constants::get_tardis_inst());
    let endpoint_funs_for_endpoint = endpoint_funs.clone();
    let endpoint = make(move |request: Request| {
        let request_bodies = request_bodies_for_endpoint.clone();
        let endpoint_funs = endpoint_funs_for_endpoint.clone();
        async move {
            let body = request.into_body().into_string().await.unwrap_or_default();
            request_bodies.lock().unwrap().push(body);
            set_provider_cache_oom(&endpoint_funs, true).await.expect("failed to enable Redis SETEX fault");
            let mut connection = endpoint_funs.cache().cmd().await.expect("failed to connect to test Redis");
            let probe: Result<String, _> = tardis::cache::cmd("SETEX").arg("provider-setex-probe").arg(60).arg("value").query_async(&mut connection).await;
            assert!(probe.is_err(), "Redis SETEX fault was not activated");
            Response::builder()
                .status(StatusCode::OK)
                .content_type("application/json")
                .body(Body::from_string(
                    r#"{"code":"200000000000","msg":"","data":{"access_token":"refreshed-access-token","token_type":"Bearer","expires_in":3600,"refresh_token":"rotated-refresh-token","scope":"iam.app.read"}}"#.to_string(),
                ))
        }
    });
    let route = Route::new().at("/cp/oauth2/token", endpoint);
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let provider_addr = listener.local_addr().unwrap();
    let acceptor = TcpAcceptor::from_tokio(listener).unwrap();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let provider_task = tokio::spawn(async move {
        Server::new_with_acceptor(acceptor)
            .run_with_graceful_shutdown(
                route,
                async move {
                    let _ = shutdown_rx.await;
                },
                Some(Duration::from_secs(1)),
            )
            .await
            .unwrap();
    });

    let cert_conf_id = IamCertOAuth2Serv::add_cert_conf(
        IamCertOAuth2Supplier::BiosIam,
        &IamCertConfOAuth2AddOrModifyReq {
            supplier: TrimString(IamCertOAuth2Supplier::BiosIam.to_string()),
            ak: TrimString("mock-client".to_string()),
            sk: TrimString("mock-secret".to_string()),
            base_url: Some(format!("http://{}", provider_addr)),
        },
        "",
        &funs,
        &TardisContext::default(),
    )
    .await?;

    let github_conf_id = IamCertOAuth2Serv::add_cert_conf(
        IamCertOAuth2Supplier::Github,
        &IamCertConfOAuth2AddOrModifyReq {
            supplier: TrimString(IamCertOAuth2Supplier::Github.to_string()),
            ak: TrimString("github-client".to_string()),
            sk: TrimString("github-secret".to_string()),
            base_url: Some("https://github.example".to_string()),
        },
        "",
        &funs,
        &TardisContext::default(),
    )
    .await?;
    let wechat_conf_id = IamCertOAuth2Serv::add_cert_conf(
        IamCertOAuth2Supplier::WechatMp,
        &IamCertConfOAuth2AddOrModifyReq {
            supplier: TrimString(IamCertOAuth2Supplier::WechatMp.to_string()),
            ak: TrimString("wechat-client".to_string()),
            sk: TrimString("wechat-secret".to_string()),
            base_url: Some("https://wechat.example".to_string()),
        },
        "",
        &funs,
        &TardisContext::default(),
    )
    .await?;
    let oauth2_kind = IamCertExtKind::OAuth2.to_string();
    assert_eq!(IamCertServ::get_cert_conf_id_by_kind_supplier(&oauth2_kind, "Github", None, &funs).await?, github_conf_id);
    assert_eq!(IamCertServ::get_cert_conf_id_by_kind_supplier(&oauth2_kind, "WechatMp", None, &funs).await?, wechat_conf_id);
    assert_eq!(IamCertServ::get_cert_conf_id_by_kind_supplier(&oauth2_kind, "BiosIam", None, &funs).await?, cert_conf_id);

    let cached_token = IamCertOAuth2TokenInfo {
        open_id: "provider-user".to_string(),
        access_token: "expired-access-token".to_string(),
        refresh_token: Some("old-refresh-token".to_string()),
        token_expires_ms: Some(60_000),
        expires_at_ms: Some(Utc::now().timestamp_millis() - 1_000),
        scope: Some("iam.app.read".to_string()),
        provider_cert_conf_id: Some(cert_conf_id.clone()),
        union_id: None,
    };
    let conf = funs.conf::<bios_iam::iam_config::IamConfig>();
    let cache_key = format!("{}{}:{}", conf.cache_key_oauth2_provider_token_, IamCertOAuth2Supplier::BiosIam, ACCOUNT_ID);
    funs.cache().set_ex(&cache_key, &TardisFuns::json.obj_to_string(&cached_token)?, 600).await?;
    let provider_grant_id = TardisFuns::crypto.digest.sha256(TardisFuns::json.obj_to_string(&(ACCOUNT_ID, &cert_conf_id, &cached_token.open_id))?)?;
    let encryption_key = vec![7; 32];
    let nonce = TardisFuns::crypto.aead.random_nonce::<Aes256Gcm>();
    let (ciphertext, _) = TardisFuns::crypto.aead.encrypt::<Aes256Gcm>(
        &encryption_key,
        provider_grant_id.as_bytes(),
        &nonce,
        TardisFuns::json.obj_to_string(&cached_token)?.as_bytes(),
    )?;
    let encrypted_token = format!("{}:{}", TardisFuns::crypto.hex.encode(nonce), TardisFuns::crypto.hex.encode(ciphertext));
    iam_oauth2_provider_grant::Entity::insert(iam_oauth2_provider_grant::ActiveModel {
        id: Set(provider_grant_id),
        account_id: Set(ACCOUNT_ID.to_string()),
        cert_conf_id: Set(cert_conf_id.clone()),
        external_subject: Set(cached_token.open_id.clone()),
        encrypted_token: Set(encrypted_token),
        update_time: Set(Utc::now().to_rfc3339()),
    })
    .exec(funs.db().raw_conn())
    .await?;

    let mut refresh_funs = iam_constants::get_tardis_inst();
    refresh_funs.begin().await?;
    let refresh_result = IamCertOAuth2Serv::get_provider_token(IamCertOAuth2Supplier::BiosIam, ACCOUNT_ID, "tenant/app", &refresh_funs).await;
    refresh_funs.rollback().await?;
    set_provider_cache_oom(&funs, false).await?;
    funs.cache().del(&format!("iam:oauth2:provider-refresh:BiosIam:{ACCOUNT_ID}")).await?;
    let refresh_error = refresh_result.as_ref().err().expect("Redis SETEX must fail after durable refresh-token rotation");
    assert_eq!(refresh_error.code, "-1");

    let cached_after_failure = funs.cache().get(&cache_key).await?.expect("the previous Redis token should remain after SETEX fails");
    let cached_after_failure: IamCertOAuth2TokenInfo = TardisFuns::json.str_to_obj(&cached_after_failure)?;
    assert_eq!(cached_after_failure.access_token, "expired-access-token");
    assert_eq!(cached_after_failure.refresh_token.as_deref(), Some("old-refresh-token"));

    {
        let request_bodies = request_bodies.lock().unwrap();
        assert_eq!(request_bodies.len(), 1);
        assert!(request_bodies[0].contains(r#""grant_type":"refresh_token""#));
        assert!(request_bodies[0].contains(r#""refresh_token":"old-refresh-token""#));
        assert!(request_bodies[0].contains(r#""scope":"iam.app.read""#));
    }

    let recovered = IamCertOAuth2Serv::get_provider_token(IamCertOAuth2Supplier::BiosIam, ACCOUNT_ID, "tenant/app", &funs).await?;
    assert_eq!(recovered.access_token, "refreshed-access-token");
    assert_eq!(recovered.refresh_token.as_deref(), Some("rotated-refresh-token"));
    assert_eq!(recovered.scope.as_deref(), Some("iam.app.read"));
    assert_eq!(recovered.provider_cert_conf_id.as_deref(), Some(cert_conf_id.as_str()));
    assert!(recovered.expires_at_ms.unwrap_or_default() > Utc::now().timestamp_millis());
    assert_eq!(request_bodies.lock().unwrap().len(), 1);

    let _ = shutdown_tx.send(());
    provider_task.await.unwrap();
    Ok(())
}
