use bios_basic::rbum::dto::rbum_item_dto::RbumItemAddReq;
use std::collections::HashMap;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    log, testcontainers, tokio,
};

mod test_reach_common;
use bios_reach::{consts::*, dto::*, invoke};
use test_reach_common::*;
#[tokio::test]
pub async fn test_ct_api() -> TardisResult<()> {
    // for debug
    // std::env::set_current_dir("./support/reach/")?;
    std::env::set_var("RUST_LOG", "info,sqlx=off,sea_orm=debug,spi_conf_namespace_test=DEBUG,bios_spi_conf=TRACE,tardis=TRACE");
    let docker = testcontainers::clients::Cli::default();
    let holder = init_tardis(&docker).await?;
    let ctx = get_test_ctx();
    let funs = get_tardis_inst();
    let client = invoke::Client::new("https://localhost:8080/reach", ctx, &funs);
    // msg template
    log::info!("add msg template");
    let template_name = random_string(16);
    let template_id = client
        .add_msg_template(&ReachMessageTemplateAddReq {
            rel_reach_channel: ReachChannelKind::Email,
            content: "content".into(),
            own_paths: ctx.own_paths.clone(),
            owner: ctx.owner.clone(),
            variables: "{}".into(),
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
        })
        .await?;
    let template = client.get_msg_template_by_id(&template_id).await?;
    assert_eq!(template.name, Some(template_name));
    // this has problem caused by sea_orm occupied ReachChannelKind::to_string(), Email will be convert to 'EMAIL' (with a pair of single quotes)
    // but simple client also use to_string
    /*     let pages = client.paginate_msg_template(
        None,
        Some(12),
        Some(ReachChannelKind::Email)
    ).await?; */
    log::info!("get msg template: {:?}", template);
    log::info!("add msg signature");
    let signature_id = client
        .add_msg_signature(&ReachMsgSignatureAddReq {
            name: "test-signature".into(),
            note: "test-note".into(),
            content: "test-signature-content".into(),
            source: "signature".into(),
            rel_reach_channel: ReachChannelKind::Email,
        })
        .await?;
    let signature = client.get_msg_signature_by_id(&signature_id).await?;
    log::info!("get msg signature: {:?}", signature);
    log::info!("add_message");
    let resp = client
        .add_message(&ReachMessageAddReq {
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
            rel_reach_channel: ReachChannelKind::Email,
            receive_kind: ReachReceiveKind::Account,
            to_res_ids: "destination@bios.dev".to_string(),
            rel_reach_msg_signature_id: signature_id,
            rel_reach_msg_template_id: template_id,
            reach_status: ReachStatusKind::Pending,
            content_replace: "{}".into(),
        })
        .await?;
    log::info!("add_message resp: {:?}", resp);
    wait_for_press();
    dbg!(resp);
    drop(holder);
    Ok(())
}
