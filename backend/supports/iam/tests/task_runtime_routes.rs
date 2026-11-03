//! 后台兑换接口必须先校验服务凭据，不能把用户身份当作服务鉴权。
use bios_iam::console_passport::api::iam_cp_cert_api::IamTaskRuntimeApi;
use tardis::{
    serde_json::{json, Value},
    tokio,
    web::{
        poem::{http::Method, Body, Endpoint, EndpointExt, Request, Route},
        poem_openapi::OpenApiService,
        uniform_error_mw::UniformError,
    },
};

#[tokio::test]
#[ignore = "requires Docker for isolated PostgreSQL, Redis and RabbitMQ"]
async fn runtime_exchange_requires_service_authentication_before_database_access() -> tardis::basic::result::TardisResult<()> {
    std::env::set_var("BIOS_OAUTH2_TASK_SERVICE_TOKEN", "test-runtime-service-secret");
    let _containers = bios_basic::test::init_test_container::init(None).await?;
    let api = Route::new().nest("/iam", OpenApiService::new(IamTaskRuntimeApi, "Runtime", "1")).with(UniformError);
    for token in [None, Some("wrong"), Some("alice")] {
        let mut req = Request::builder().method(Method::POST).uri("/iam/internal/oauth2/task-grants/exchange".parse().unwrap()).header("Content-Type", "application/json");
        if let Some(token) = token {
            req = req.header("X-Bios-Task-Service-Token", token);
        }
        let response = api
            .get_response(
                req.body(
                    Body::from_json(&json!({
                        "grant_id": "grant", "task_id": "binding/item", "own_paths": "tenant/app"
                    }))
                    .unwrap(),
                ),
            )
            .await;
        let body: Value = response.into_body().into_json().await.unwrap();
        assert!(body["code"].as_str().unwrap().starts_with("401"), "{body}");
    }
    Ok(())
}
