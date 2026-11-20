use bios_basic::rbum::rbum_enumeration::RbumCertStatusKind;
use bios_basic::test::init_test_container;
use bios_iam::basic::dto::iam_cert_conf_dto::IamCertConfOAuth2AddOrModifyReq;
use bios_iam::basic::serv::iam_cert_oauth2_serv::IamCertOAuth2Serv;
use bios_iam::basic::serv::iam_cert_oauth2_service_serv::IamCertOAuth2ServiceServ;
use bios_iam::basic::serv::iam_cert_serv::IamCertServ;
use bios_iam::iam_constants;
use bios_iam::iam_enumeration::{IamCertKernelKind, IamCertOAuth2Supplier};
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::tokio;
async fn start_mock_provider() -> TardisResult<(String, tardis::tokio::task::JoinHandle<()>, tardis::tokio::sync::oneshot::Sender<()>)> {
    use tardis::web::poem::{endpoint::make, http::StatusCode, listener::TcpAcceptor, Body, Request, Response, Route, Server};

    let token_endpoint = make(|_request: Request| async move {
        Response::builder()
            .status(StatusCode::OK)
            .content_type("application/json")
            .body(Body::from_string(
                r#"{"code":"200000000000","msg":"","data":{"access_token":"provider-access-token","token_type":"Bearer","expires_in":3600,"refresh_token":"provider-refresh-token","scope":"iam.userinfo.read"}}"#
                    .to_string(),
            ))
    });
    let userinfo_endpoint = make(|_request: Request| async move {
        Response::builder()
            .status(StatusCode::OK)
            .content_type("application/json")
            .body(Body::from_string(
                r#"{"code":"200000000000","msg":"","data":{"provider":"bios-iam","sub":"provider-account","tenant_id":"","name":"Provider Display Name","user_name":"provider_login","mail":"provider@example.com","phone":"13800000009","employee_no":null,"id_card_no":null,"disabled":false}}"#
                    .to_string(),
            ))
    });
    let route = Route::new().at("/cp/oauth2/token", token_endpoint).at("/cp/oauth2/userinfo", userinfo_endpoint);
    let listener = tardis::tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let addr = listener.local_addr().unwrap();
    let acceptor = TcpAcceptor::from_tokio(listener).unwrap();
    let (shutdown_tx, shutdown_rx) = tardis::tokio::sync::oneshot::channel();
    let handle = tardis::tokio::spawn(async move {
        Server::new_with_acceptor(acceptor)
            .run_with_graceful_shutdown(
                route,
                async move {
                    let _ = shutdown_rx.await;
                },
                Some(std::time::Duration::from_secs(1)),
            )
            .await
            .unwrap();
    });
    Ok((format!("http://{addr}"), handle, shutdown_tx))
}

#[tokio::test]
async fn first_authorization_syncs_login_name_phone_and_mail_without_activation() -> TardisResult<()> {
    let _containers = init_test_container::init(None).await?;
    let _ = bios_iam::iam_initializer::init_db(iam_constants::get_tardis_inst()).await?;
    let funs = iam_constants::get_tardis_inst();
    let (provider_base_url, provider_task, shutdown_tx) = start_mock_provider().await?;

    IamCertOAuth2Serv::add_cert_conf(
        IamCertOAuth2Supplier::BiosIam,
        &IamCertConfOAuth2AddOrModifyReq {
            supplier: TrimString(IamCertOAuth2Supplier::BiosIam.to_string()),
            ak: TrimString("first-auth-client".to_string()),
            sk: TrimString("first-auth-secret".to_string()),
            base_url: Some(provider_base_url),
        },
        "",
        &funs,
        &TardisContext::default(),
    )
    .await?;

    let (account_id, _) = IamCertOAuth2Serv::get_or_add_account(IamCertOAuth2Supplier::BiosIam, "auth-code", "", &funs).await?;

    let ctx = TardisContext {
        own_paths: "".to_string(),
        owner: account_id.clone(),
        ..Default::default()
    };
    let userpwd = IamCertServ::get_kernel_cert(&account_id, &IamCertKernelKind::UserPwd, &funs, &ctx).await?;
    assert_eq!(userpwd.ak, "provider_login");

    let phone = IamCertServ::get_kernel_cert(&account_id, &IamCertKernelKind::PhoneVCode, &funs, &ctx).await?;
    assert_eq!(phone.ak, "13800000009");
    assert_eq!(phone.status, RbumCertStatusKind::Enabled, "synced phone must not go through local activation");

    let mail = IamCertServ::get_kernel_cert(&account_id, &IamCertKernelKind::MailVCode, &funs, &ctx).await?;
    assert_eq!(mail.ak, "provider@example.com");
    assert_eq!(mail.status, RbumCertStatusKind::Enabled);

    let userinfo = IamCertOAuth2ServiceServ::build_userinfo_by_account_id(&account_id, None, &funs).await?;
    assert_eq!(userinfo.name, "Provider Display Name");
    assert_eq!(userinfo.user_name.as_deref(), Some("provider_login"));

    let _ = shutdown_tx.send(());
    let _ = provider_task.await;
    Ok(())
}
