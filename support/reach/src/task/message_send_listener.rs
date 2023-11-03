use std::{collections::HashSet, sync::Arc};

use crate::{domain::*, dto::*, reach_config::ReachConfig, reach_consts::*, reach_init::get_reach_send_channel_map, reach_send_channel::*, serv::*};
use bios_basic::rbum::{helper::rbum_scope_helper, serv::rbum_crud_serv::RbumCrudOperation};
use bios_sdk_invoke::clients::iam_client::IamClient;
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
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

pub struct SingleMessageSendTask<'m> {
    model: &'m message::Model,
    status: ReachStatusKind,
    funs: Arc<TardisFunsInst>,
    // channel: &'static SendChannelMap,
}

impl SingleMessageSendTask<'_> {
    async fn update_status(&mut self, new_status: ReachStatusKind) -> TardisResult<()> {
        if !ReachMessageServ::update_status(&self.model.id, self.status, new_status, &self.funs).await? {
            let message = format!(
                "message status conflict, id: {}, old_status: {:?}, new_status: {:?}",
                self.model.id, self.status, new_status
            );
            return Err(TardisError::conflict(&message, "409-reach-message-status-conflict"));
        }
        self.status = new_status;
        Ok(())
    }
}

impl MessageSendListener {
    pub async fn send_single_message(&self, task: &SingleMessageSendTask<'_>, funs: &TardisFunsInst) -> TardisResult<()> {
        let db = funs.db();
        let message = task.model;
        let mut task = self.new_pending_task(message);
        let cfg = self.funs.conf::<ReachConfig>();
        let ctx = TardisContext {
            own_paths: message.own_paths.clone(),
            ..Default::default()
        };
        let content_replace: ContentReplace = message.content_replace.parse()?;
        let iam_client = Arc::new(IamClient::new(
            &cfg.iam_get_account,
            funs,
            &ctx,
            cfg.invoke.module_urls.get("iam").expect("missing iam base url"),
        ));

        let Some(template) = db
            .get_dto::<message_template::Model>(
                Query::select()
                    .columns(message_template::Column::iter())
                    .from(message_template::Entity)
                    .and_where(message_template::Column::Id.eq(&message.rel_reach_msg_template_id)),
            )
            .await?
        else {
            tardis::tracing::warn!("[Bios.Reach] missing message template");
            return Err(TardisError::not_found("missing message template", "404-reach-message-template-not-found"));
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

        let cert_key = match message.rel_reach_channel {
            ReachChannelKind::Sms => IAM_KEY_PHONE_V_CODE,
            ReachChannelKind::Email => IAM_KEY_MAIL_V_CODE,
            _ => {
                return Err(TardisError::conflict(
                    &format!("channel [{channel}] not yet implemented", channel = message.rel_reach_channel),
                    "409-reach-message-unimplemented-channel",
                ));
            }
        };
        let owner_path = rbum_scope_helper::get_pre_paths(RBUM_SCOPE_LEVEL_TENANT as i16, &message.own_paths).unwrap_or_default();
        let mut to = HashSet::new();
        for account_id in message.to_res_ids.split(ACCOUNT_SPLIT) {
            if let Ok(mut resp) = iam_client.get_account(account_id, &owner_path).await {
                let Some(res_id) = resp.certs.remove(cert_key) else {
                    log::warn!(
                        "[Reach] Notify {chan} channel send error, missing [{cert_key}] parameters, resp: {resp:?}",
                        chan = message.rel_reach_channel
                    );
                    continue;
                };
                to.insert(res_id);
            } else {
                log::warn!("[Reach] iam get account info error, account_id: {account_id}")
            }
        }

        let start_time = Utc::now();
        let result = match message.receive_kind {
            ReachReceiveKind::Account => {
                task.update_status(ReachStatusKind::Sending).await?;
                self.channel.send(message.rel_reach_channel, &template, &content_replace, &to).await
            }
            kind => {
                let message = format!("receive kind {kind:?} not implemented");
                Err(TardisError::not_implemented(&message, "501-reach-message-receive-kind-not-implemented"))
            }
        };
        let end_time = Utc::now();

        let failure = result.is_err();
        let fail_message = result.err().map(|e| e.to_string()).unwrap_or_default();
        if failure {
            task.update_status(ReachStatusKind::Fail).await?;
        } else {
            task.update_status(ReachStatusKind::SendSuccess).await?;
        };
        for id in to {
            ReachMessageLogServ::add_rbum(
                &mut ReachMsgLogAddReq {
                    rbum_add_req: Default::default(),
                    dnd_time: Default::default(),
                    rel_account_id: id,
                    dnd_strategy: ReachDndStrategyKind::Ignore,
                    start_time,
                    end_time,
                    failure,
                    fail_message: fail_message.clone(),
                    rel_reach_message_id: message.id.clone(),
                },
                &self.funs,
                &ctx,
            )
            .await?;
        }
        Ok(())
    }

    pub async fn run(&self) -> TardisResult<()> {
        let funs = get_tardis_inst();
        let db = funs.db();
        let _sync = self.sync.lock().await;
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
            let mut send_task = self.new_pending_task(&message);
            if self.send_single_message(&send_task, &funs).await.is_err() {
                send_task.update_status(ReachStatusKind::Fail).await?;
            }
        }
        Ok(())
    }

    pub fn new_pending_task<'t>(&'t self, model: &'t message::Model) -> SingleMessageSendTask {
        SingleMessageSendTask {
            model,
            status: ReachStatusKind::Pending,
            funs: self.funs.clone(),
            // channel: self.channel,
        }
    }
}
