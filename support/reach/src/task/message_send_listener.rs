use std::{collections::HashSet, sync::Arc};

use crate::{domain::*, dto::*, reach_config::ReachConfig, reach_consts::*, reach_init::get_reach_send_channel_map, reach_send_channel::*, serv::*};
use bios_basic::rbum::{helper::rbum_scope_helper, serv::rbum_crud_serv::RbumCrudOperation};
use bios_sdk_invoke::clients::iam_client::IamClient;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    chrono::Utc,
    db::sea_orm::{sea_query::Query, *},
    log, tokio, TardisFunsInst,
};

#[derive(Clone)]
pub struct MessageSendListener {
    sync: Arc<tokio::sync::Mutex<()>>,
    funs: Arc<TardisFunsInst>,
    channel: &'static SendChannelMap,
}

impl Default for MessageSendListener {
    fn default() -> Self {
        Self {
            sync: Default::default(),
            funs: get_tardis_inst().into(),
            channel: get_reach_send_channel_map(),
        }
    }
}

impl MessageSendListener {
    async fn execute_send_account(&self, message: message::Model, template: message_template::Model, _signarure: Option<message_signature::Model>) -> TardisResult<()> {
        let content_replace: ContentReplace = message.content_replace.parse()?;
        let cfg = self.funs.conf::<ReachConfig>();
        let _lock = self.sync.lock().await;
        let ctx = TardisContext {
            own_paths: message.own_paths.clone(),
            ..Default::default()
        };
        let iam_client = Arc::new(IamClient::new(
            &cfg.iam_get_account,
            &self.funs,
            &ctx,
            cfg.invoke.module_urls.get("iam").expect("missing iam base url"),
        ));
        // if not pending status, this task may be excuted by other nodes, just return
        if !ReachMessageServ::update_status(&message.id, ReachStatusKind::Pending, ReachStatusKind::Sending, &self.funs, &ctx).await? {
            return Ok(());
        }
        let mut to = HashSet::new();
        let start_time = Utc::now();
        let owner_path = rbum_scope_helper::get_pre_paths(RBUM_SCOPE_LEVEL_TENANT as i16, &message.own_paths).unwrap_or_default();
        let cert_key = match message.rel_reach_channel {
            ReachChannelKind::Sms => IAM_KEY_PHONE_V_CODE,
            ReachChannelKind::Email => IAM_KEY_MAIL_V_CODE,
            _ => {
                // unsupported
                ReachMessageServ::update_status(&message.id, ReachStatusKind::Pending, ReachStatusKind::Fail, &self.funs, &ctx).await?;
                return Ok(());
            }
        };
        for account_id in message.to_res_ids.split(ACCOUNT_SPLIT) {
            if let Ok(mut resp) = iam_client.get_account(account_id, &owner_path).await {
                match message.rel_reach_channel {
                    ReachChannelKind::Sms => {
                        let Some(res_id) = resp.certs.remove(cert_key) else {
                            log::warn!("[Reach] Notify Phone channel send error, missing [{cert_key}] parameters, resp: {resp:?}");
                            continue;
                        };
                        to.insert(res_id);
                    }
                    ReachChannelKind::Email => {}
                    _ => {
                        continue;
                    }
                }
            } else {
                log::warn!("[Reach] iam get account info error, account_id: {account_id}")
            }
        }
        match self.channel.send(message.rel_reach_channel, &template, &content_replace, &to).await {
            Ok(_) => {
                ReachMessageServ::update_status(&message.id, ReachStatusKind::Sending, ReachStatusKind::SendSuccess, &self.funs, &ctx).await?;
                for rel_account_id in to {
                    ReachMessageLogServ::add_rbum(
                        &mut ReachMsgLogAddReq {
                            rbum_add_req: Default::default(),
                            dnd_time: Default::default(),
                            rel_account_id,
                            dnd_strategy: ReachDndStrategyKind::Ignore,
                            start_time,
                            end_time: Utc::now(),
                            failure: false,
                            fail_message: Default::default(),
                            rel_reach_message_id: message.id.clone(),
                        },
                        &self.funs,
                        &ctx,
                    )
                    .await?;
                }
            }
            Err(e) => {
                ReachMessageServ::update_status(&message.id, ReachStatusKind::Sending, ReachStatusKind::Fail, &self.funs, &ctx).await?;
                for rel_account_id in to {
                    ReachMessageLogServ::add_rbum(
                        &mut ReachMsgLogAddReq {
                            rbum_add_req: Default::default(),
                            dnd_time: Default::default(),
                            rel_account_id,
                            dnd_strategy: ReachDndStrategyKind::Ignore,
                            start_time,
                            end_time: Utc::now(),
                            failure: true,
                            fail_message: e.to_string(),
                            rel_reach_message_id: message.id.clone(),
                        },
                        &self.funs,
                        &ctx,
                    )
                    .await?;
                }

                return Err(e);
            }
        }
        Ok(())
    }
    pub async fn run(&self) -> TardisResult<()> {
        let funs = get_tardis_inst();
        let db = funs.db();
        let messages: Vec<message::Model> = db
            .find_dtos(
                Query::select()
                    .columns(message::Column::iter())
                    .from(message::Entity)
                    .and_where(message::Column::ReachStatus.eq(ReachStatusKind::Pending))
                    .and_where(message::Column::ReceiveKind.eq(ReachReceiveKind::Account)),
            )
            .await?;
        for message in messages {
            let Some(template) = db
                .get_dto::<message_template::Model>(
                    Query::select()
                        .columns(message_template::Column::iter())
                        .from(message_template::Entity)
                        .and_where(message_template::Column::Id.eq(&message.rel_reach_msg_template_id)),
                )
                .await?
            else {
                continue;
            };
            // signature is not necessary now
            // let signature = db
            //     .get_dto::<message_signature::Model>(
            //         Query::select()
            //             .columns(message_signature::Column::iter())
            //             .from(message_signature::Entity)
            //             .and_where(message_signature::Column::Id.eq(&message.rel_reach_msg_signature_id)),
            //     )
            //     .await?;
            match message.receive_kind {
                ReachReceiveKind::Account => {
                    let _res = self.execute_send_account(message, template, None).await;
                }
                ReachReceiveKind::Tenant => {
                    // do nothing
                }
                ReachReceiveKind::Role => {
                    // do nothing
                }
                ReachReceiveKind::App => {
                    // do nothing
                }
            }
        }
        Ok(())
    }
}
