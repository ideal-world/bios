use std::collections::HashMap;

use bios_basic::rbum::dto::rbum_cert_dto::RbumCertSummaryWithSkResp;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_sdk_invoke::clients::reach_client::ReachMessageAddSendTaskReq;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::chrono::{DateTime, Datelike, FixedOffset, TimeZone, Utc};
use tardis::TardisFunsInst;

use crate::basic::dto::iam_cert_dto::{IamCcThirdPartyCertExpiryNotifyItemResp, IamCcThirdPartyCertExpiryNotifyResp, IamCcThirdPartyCertExpiryNotifySkippedItemResp};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::clients::sms_client::SmsClient;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::iam_config::IamConfig;
use crate::iam_enumeration::IamCertKernelKind;

/// 三方凭证到期提醒：在到期前 14 / 7 / 3 / 1 天各通知一次
const NOTIFY_REMAINING_DAYS: [i64; 4] = [14, 7, 3, 1];

/// 去重缓存 TTL（略大于 24 小时，避免同日重复发送）
const NOTIFY_DEDUP_CACHE_SEC: u64 = 60 * 60 * 26;

struct PendingNotifyCert {
    cert_id: String,
    account_id: String,
    supplier: String,
    end_time: DateTime<Utc>,
    remaining_days: i64,
}

pub struct IamCcCertExpiryNotifyServ;

impl IamCcCertExpiryNotifyServ {
    /// 扫描即将到期的三方凭证，向已绑定手机号的账号发送短信提醒。
    ///
    /// 同一账号同一天仅发送一条短信；若有多张凭证同时命中提醒节点，取剩余天数最小者。
    pub async fn notify_expiring_third_party_certs(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamCcThirdPartyCertExpiryNotifyResp> {
        let iam_conf = funs.conf::<IamConfig>();
        if iam_conf.third_party_cert_expiry_reach_msg_signature_id.is_empty() || iam_conf.third_party_cert_expiry_reach_msg_template_id.is_empty() {
            return Err(funs.err().bad_request(
                "iam_cc_cert_expiry_notify",
                "notify_expiring_third_party_certs",
                "third_party_cert_expiry reach msg signature/template id is not configured",
                "400-iam-cert-expiry-notify-config-missing",
            ));
        }

        let gmt_plus_8 = FixedOffset::east_opt(8 * 3600).expect("GMT+8 timezone offset is valid");
        let today = Utc::now().with_timezone(&gmt_plus_8).date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();
        let certs = IamCertServ::find_3th_kind_cert(None, None, false, None, funs, ctx).await?;

        let mut sent = vec![];
        let mut skipped = vec![];
        let mut account_best: HashMap<String, PendingNotifyCert> = HashMap::new();

        for cert in certs {
            let remaining_days = (cert.end_time.date_naive() - today).num_days();
            if !NOTIFY_REMAINING_DAYS.contains(&remaining_days) {
                continue;
            }

            let pending = Self::pending_from_cert(cert, remaining_days);
            match account_best.get(&pending.account_id) {
                Some(best) if pending.remaining_days < best.remaining_days => {
                    skipped.push(IamCcThirdPartyCertExpiryNotifySkippedItemResp {
                        cert_id: best.cert_id.clone(),
                        account_id: best.account_id.clone(),
                        reason: format!(
                            "superseded by cert {} with smaller remaining_days={}",
                            pending.cert_id, pending.remaining_days
                        ),
                    });
                    account_best.insert(pending.account_id.clone(), pending);
                }
                Some(best) => {
                    skipped.push(IamCcThirdPartyCertExpiryNotifySkippedItemResp {
                        cert_id: pending.cert_id,
                        account_id: pending.account_id,
                        reason: format!(
                            "superseded by cert {} with smaller or equal remaining_days={}",
                            best.cert_id, best.remaining_days
                        ),
                    });
                }
                None => {
                    account_best.insert(pending.account_id.clone(), pending);
                }
            }
        }

        for pending in account_best.into_values() {
            let account_id = pending.account_id.clone();
            let dedup_key = format!(
                "{}{}:{}",
                iam_conf.cache_key_third_party_cert_expiry_notify_,
                account_id,
                today_str
            );
            if funs.cache().exists(&dedup_key).await? {
                skipped.push(IamCcThirdPartyCertExpiryNotifySkippedItemResp {
                    cert_id: pending.cert_id.clone(),
                    account_id: account_id.clone(),
                    reason: "account already notified today".to_string(),
                });
                continue;
            }

            let account_ctx = match IamAccountServ::is_global_account_context(account_id.as_str(), funs, ctx).await {
                Ok(c) => c,
                Err(e) => {
                    skipped.push(IamCcThirdPartyCertExpiryNotifySkippedItemResp {
                        cert_id: pending.cert_id.clone(),
                        account_id: account_id.clone(),
                        reason: format!("account context error: {e}"),
                    });
                    continue;
                }
            };

            if IamCertServ::get_kernel_cert(account_id.as_str(), &IamCertKernelKind::PhoneVCode, funs, &account_ctx)
                .await
                .is_err()
            {
                skipped.push(IamCcThirdPartyCertExpiryNotifySkippedItemResp {
                    cert_id: pending.cert_id.clone(),
                    account_id: account_id.clone(),
                    reason: "account has no phone cert".to_string(),
                });
                continue;
            }

            let Some(account) = IamAccountServ::find_one_item(
                &IamAccountFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        own_paths: Some("".to_string()),
                        ids: Some(vec![account_id.clone()]),
                        enabled: Some(true),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                &account_ctx,
            )
            .await?
            else {
                skipped.push(IamCcThirdPartyCertExpiryNotifySkippedItemResp {
                    cert_id: pending.cert_id.clone(),
                    account_id: account_id.clone(),
                    reason: "account not found".to_string(),
                });
                continue;
            };

            let mut replace = HashMap::new();
            replace.insert("end_time".to_string(), Some(Self::format_end_time(pending.end_time)));
            replace.insert("remaining_days".to_string(), Some(pending.remaining_days.to_string()));
            replace.insert("username".to_string(), Some(account.name));

            SmsClient::add_send_task(
                &ReachMessageAddSendTaskReq {
                    rel_reach_channel: "SMS".to_string(),
                    receive_kind: "ACCOUNT".to_string(),
                    to_res_ids: vec![account_id.clone()],
                    rel_reach_msg_signature_id: iam_conf.third_party_cert_expiry_reach_msg_signature_id.clone(),
                    rel_reach_msg_template_id: iam_conf.third_party_cert_expiry_reach_msg_template_id.clone(),
                    replace,
                },
                funs,
                ctx,
            )
            .await?;

            funs.cache().set_ex(&dedup_key, "1", NOTIFY_DEDUP_CACHE_SEC).await?;

            sent.push(IamCcThirdPartyCertExpiryNotifyItemResp {
                cert_id: pending.cert_id,
                account_id,
                supplier: pending.supplier,
                end_time: pending.end_time,
                remaining_days: pending.remaining_days,
            });
        }

        Ok(IamCcThirdPartyCertExpiryNotifyResp { sent, skipped })
    }

    /// 扫描今日已到期的三方凭证，向已绑定手机号的账号发送短信提醒。
    ///
    /// 判定条件：到期日等于当天且当前时间已过 `end_time`；同一账号同一天仅发送一条。
    pub async fn notify_expired_today_third_party_certs(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamCcThirdPartyCertExpiryNotifyResp> {
        let iam_conf = funs.conf::<IamConfig>();
        if iam_conf.third_party_cert_expired_reach_msg_signature_id.is_empty() || iam_conf.third_party_cert_expired_reach_msg_template_id.is_empty() {
            return Err(funs.err().bad_request(
                "iam_cc_cert_expiry_notify",
                "notify_expired_today_third_party_certs",
                "third_party_cert_expired reach msg signature/template id is not configured",
                "400-iam-cert-expired-notify-config-missing",
            ));
        }

        let gmt_plus_8 = FixedOffset::east_opt(8 * 3600).expect("GMT+8 timezone offset is valid");
        let now = Utc::now().with_timezone(&gmt_plus_8);
        let today = now.date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();
        let certs = IamCertServ::find_3th_kind_cert(None, None, false, None, funs, ctx).await?;

        let mut sent = vec![];
        let mut skipped = vec![];
        let mut account_best: HashMap<String, PendingNotifyCert> = HashMap::new();

        for cert in certs {
            if cert.end_time.date_naive() != today || now < cert.end_time {
                continue;
            }
            let remaining_days = (cert.end_time.date_naive() - today).num_days();
            let pending = Self::pending_from_cert(cert, remaining_days);
            match account_best.get(&pending.account_id) {
                Some(best) if pending.end_time < best.end_time => {
                    skipped.push(IamCcThirdPartyCertExpiryNotifySkippedItemResp {
                        cert_id: best.cert_id.clone(),
                        account_id: best.account_id.clone(),
                        reason: format!("superseded by cert {} with earlier end_time", pending.cert_id),
                    });
                    account_best.insert(pending.account_id.clone(), pending);
                }
                Some(best) => {
                    skipped.push(IamCcThirdPartyCertExpiryNotifySkippedItemResp {
                        cert_id: pending.cert_id,
                        account_id: pending.account_id,
                        reason: format!("superseded by cert {} with earlier or equal end_time", best.cert_id),
                    });
                }
                None => {
                    account_best.insert(pending.account_id.clone(), pending);
                }
            }
        }

        for pending in account_best.into_values() {
            let account_id = pending.account_id.clone();
            let dedup_key = format!(
                "{}{}:{}",
                iam_conf.cache_key_third_party_cert_expired_notify_,
                account_id,
                today_str
            );
            if funs.cache().exists(&dedup_key).await? {
                skipped.push(IamCcThirdPartyCertExpiryNotifySkippedItemResp {
                    cert_id: pending.cert_id.clone(),
                    account_id: account_id.clone(),
                    reason: "account already notified today".to_string(),
                });
                continue;
            }

            let account_ctx = match IamAccountServ::is_global_account_context(account_id.as_str(), funs, ctx).await {
                Ok(c) => c,
                Err(e) => {
                    skipped.push(IamCcThirdPartyCertExpiryNotifySkippedItemResp {
                        cert_id: pending.cert_id.clone(),
                        account_id: account_id.clone(),
                        reason: format!("account context error: {e}"),
                    });
                    continue;
                }
            };

            if IamCertServ::get_kernel_cert(account_id.as_str(), &IamCertKernelKind::PhoneVCode, funs, &account_ctx)
                .await
                .is_err()
            {
                skipped.push(IamCcThirdPartyCertExpiryNotifySkippedItemResp {
                    cert_id: pending.cert_id.clone(),
                    account_id: account_id.clone(),
                    reason: "account has no phone cert".to_string(),
                });
                continue;
            }

            let Some(account) = IamAccountServ::find_one_item(
                &IamAccountFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        own_paths: Some("".to_string()),
                        ids: Some(vec![account_id.clone()]),
                        enabled: Some(true),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                &account_ctx,
            )
            .await?
            else {
                skipped.push(IamCcThirdPartyCertExpiryNotifySkippedItemResp {
                    cert_id: pending.cert_id.clone(),
                    account_id: account_id.clone(),
                    reason: "account not found".to_string(),
                });
                continue;
            };

            let mut replace = HashMap::new();
            replace.insert("end_time".to_string(), Some(Self::format_end_time(pending.end_time)));
            replace.insert("username".to_string(), Some(account.name));

            SmsClient::add_send_task(
                &ReachMessageAddSendTaskReq {
                    rel_reach_channel: "SMS".to_string(),
                    receive_kind: "ACCOUNT".to_string(),
                    to_res_ids: vec![account_id.clone()],
                    rel_reach_msg_signature_id: iam_conf.third_party_cert_expired_reach_msg_signature_id.clone(),
                    rel_reach_msg_template_id: iam_conf.third_party_cert_expired_reach_msg_template_id.clone(),
                    replace,
                },
                funs,
                ctx,
            )
            .await?;

            funs.cache().set_ex(&dedup_key, "1", NOTIFY_DEDUP_CACHE_SEC).await?;

            sent.push(IamCcThirdPartyCertExpiryNotifyItemResp {
                cert_id: pending.cert_id,
                account_id,
                supplier: pending.supplier,
                end_time: pending.end_time,
                remaining_days: pending.remaining_days,
            });
        }

        Ok(IamCcThirdPartyCertExpiryNotifyResp { sent, skipped })
    }

    fn pending_from_cert(cert: RbumCertSummaryWithSkResp, remaining_days: i64) -> PendingNotifyCert {
        PendingNotifyCert {
            cert_id: cert.id,
            account_id: cert.rel_rbum_id,
            supplier: cert.supplier,
            end_time: cert.end_time,
            remaining_days,
        }
    }

    fn format_end_time(end_time: DateTime<Utc>) -> String {
        end_time.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}
