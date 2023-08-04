use bios_basic::rbum::dto::rbum_item_dto::RbumItemAddReq;
use std::collections::HashMap;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log,
    serde_json::json,
    testcontainers, tokio,
};

mod test_reach_common;
use bios_reach::{consts::*, dto::*, invoke};
use test_reach_common::*;
#[tokio::test]
pub async fn test_ct_api() -> TardisResult<()> {
    // for debug
    // std::env::set_current_dir("./support/reach/")?;
    std::env::set_var("RUST_LOG", "info,sqlx=off,sea_orm=debug,tardis=TRACE");
    // std::env::set_var("RUST_LOG", "test_ct=info");
    let docker = testcontainers::clients::Cli::default();
    let holder = init_tardis(&docker).await?;
    let mut sms_mocker = HwSmsMockServer::new("127.0.0.1:8081");
    sms_mocker.init().await;
    let ctx = get_test_ctx();
    let funs = get_tardis_inst();
    let client = invoke::Client::new("https://localhost:8080/reach", ctx, &funs);

    let template_id = {
        // msg template apis
        let template_name = random_string(16);
        let message_add_req = ReachMessageTemplateAddReq {
            rel_reach_channel: ReachChannelKind::Sms,
            content: "hello {name}, your code is {code}".into(),
            own_paths: ctx.own_paths.clone(),
            owner: ctx.owner.clone(),
            variables: "name,code".into(),
            level_kind: ReachLevelKind::Normal,
            topic: "hellow".to_string(),
            timeout_sec: 6000,
            timeout_strategy: ReachTimeoutStrategyKind::Ignore,
            kind: ReachTemplateKind::Vcode,
            rel_reach_verify_code_strategy_id: "strategy-id".into(),
            sms_template_id: "sms-tempalte-id".into(),
            sms_signature: "sms-signature".into(),
            sms_from: "reach@bios.dev".into(),
            scope_level: Some(0),
            code: Some("test-code".into()),
            name: Some(template_name.clone()),
            note: "test-note".into(),
            icon: "test-icon".into(),
            ..Default::default()
        };
        // msg template
        log::info!("add msg template");
        let template_id = client.add_msg_template(&message_add_req).await?;
        let template = client.get_msg_template_by_id(&template_id).await?;
        assert_eq!(template.name, Some(template_name));
        let pages = client.paginate_msg_template(None, Some(10), Some("Email")).await?;
        assert_eq!(pages.total_size, 0);
        let pages = client.paginate_msg_template(None, Some(10), Some("Sms")).await?;
        assert_eq!(pages.total_size, 1);
        let all_sms_templates = client.find_msg_template(Some("Sms")).await?;
        assert_eq!(all_sms_templates.len(), 1);
        let all_templates = client.find_msg_template(None).await?;
        assert_eq!(all_templates.len(), 1);
        log::info!("modify msg template");
        client
            .modify_msg_template(
                &template_id,
                &ReachMessageTemplateModifyReq {
                    name: Some("test-template-modified".into()),
                    ..Default::default()
                },
            )
            .await?;
        let template = client.get_msg_template_by_id(&template_id).await?;
        assert_eq!(template.name, Some("test-template-modified".into()));
        log::info!("delete msg template");
        let ok = client.delete_msg_template(&template_id).await?;
        assert!(ok);
        let ok = client.delete_msg_template(&template_id).await?;
        assert!(!ok);
        let is_not_found = client.get_msg_template_by_id(&template_id).await.is_err_and(|e| e.code.contains("404"));
        assert!(is_not_found);
        log::info!("re-add msg template");
        client.add_msg_template(&message_add_req).await?
    };

    let signature_id = {
        // msg signature apis
        let sigadd_req = ReachMsgSignatureAddReq {
            name: "test-signature".into(),
            note: "test-note".into(),
            content: "test-signature-content".into(),
            source: "signature".into(),
            rel_reach_channel: ReachChannelKind::Sms,
        };
        log::info!("add msg signature");
        let signature_id = client.add_msg_signature(&sigadd_req).await?;
        let signature = client.get_msg_signature_by_id(&signature_id).await?;
        log::info!("get msg signature: {:?}", signature);
        let pages = client.paginate_msg_signature(None, Some(10)).await?;
        assert_eq!(pages.total_size, 1);
        let all_signatures = client.find_msg_signature().await?;
        assert_eq!(all_signatures.len(), 1);
        log::info!("modify msg signature");
        client
            .modify_msg_signature(
                &signature_id,
                &ReachMsgSignatureModifyReq {
                    name: Some("test-signature-modified".into()),
                    ..Default::default()
                },
            )
            .await?;
        let signature = client.get_msg_signature_by_id(&signature_id).await?;
        assert_eq!(signature.name, "test-signature-modified");
        log::info!("delete msg signature");
        let ok = client.delete_msg_signature(&signature_id).await?;
        assert!(ok);
        let ok = client.delete_msg_signature(&signature_id).await?;
        assert!(!ok);
        log::info!("re-add msg signature");
        client.add_msg_signature(&sigadd_req).await?
    };
    // send message
    {
        log::info!("send message");
        let code = random_string(6);
        let to_name = "Bob";
        let _resp = client.general_send(to_name, &template_id, &[("name".to_owned(), to_name.to_owned()), ("code".to_owned(), code.clone())].into()).await?;
        let msg = sms_mocker.get_latest_message(to_name).await.expect("message not found");
        assert_eq!(msg, format!("hello {to_name}, your code is {code}"));
    }

    // add messages

    {
        let name = "cindy";
        let code = random_string(6);
        let add_message_req = ReachMessageAddReq {
            rbum_item_add_req: RbumItemAddReq {
                id: None,
                code: None,
                name: "test-msg".into(),
                rel_rbum_kind_id: RBUM_KIND_CODE_REACH_MESSAGE.into(),
                rel_rbum_domain_id: DOMAIN_CODE.into(),
                scope_level: None,
                disabled: None,
            },
            from_res: "from-res".to_string(),
            rel_reach_channel: ReachChannelKind::Sms,
            receive_kind: ReachReceiveKind::Account,
            to_res_ids: [name].join(","),
            rel_reach_msg_signature_id: signature_id.clone(),
            rel_reach_msg_template_id: template_id.clone(),
            reach_status: ReachStatusKind::Pending,
            content_replace: json!({
                "name": name,
                "code": code
            })
            .to_string(),
        };
        // msg send api
        log::info!("add_message");
        let resp = client.add_message(&add_message_req).await?;
        // let's waiting for 5 second to see if the message is sent
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        let msg = sms_mocker.get_latest_message(name).await;

        log::info!("latest message for {name}: {:?}", msg);
    }

    wait_for_press();
    drop(holder);
    Ok(())
}
