use bios_basic::process::task_processor::TaskProcessor;
use bios_iam::basic::dto::iam_cert_conf_dto::IamCertConfLdapAddOrModifyReq;
use bios_iam::basic::dto::iam_cert_dto::IamThirdIntegrationConfigDto;
use bios_iam::basic::dto::iam_filer_dto::IamAccountFilterReq;
use bios_iam::basic::serv::iam_account_serv::IamAccountServ;
use bios_iam::basic::serv::iam_cert_ldap_serv::IamCertLdapServ;
use bios_iam::basic::serv::iam_cert_serv::IamCertServ;
use std::time::Duration;

use bios_iam::iam_constants;
use bios_iam::iam_enumeration::{IamCertExtKind, WayToAdd, WayToDelete};
use ldap3::log::info;

use bios_iam::iam_config::IamConfig;
use tardis::basic::dto::TardisContext;
use tardis::log::error;
use tardis::tokio;
use tardis::tokio::time::sleep;

pub async fn test(ldap_account_num: u64, conf_ldap_add_or_modify_req: IamCertConfLdapAddOrModifyReq, admin_ctx: &TardisContext) {
    let funs = iam_constants::get_tardis_inst();

    let account_page = IamAccountServ::paginate_account_summary_aggs(
        &IamAccountFilterReq {
            basic: Default::default(),
            ..Default::default()
        },
        true,
        true,
        1,
        50,
        None,
        None,
        &funs,
        admin_ctx,
    )
    .await
    .unwrap();
    let account_offset = account_page.records.len();

    //不能开启事务 iam_sync_ldap_user_to_iam 这个方法里有自己的事务
    info!("【test ldap conf curd】");
    let ldap_cert_conf = IamCertLdapServ::get_cert_conf_by_ctx(&funs, admin_ctx).await.unwrap();
    assert!(ldap_cert_conf.is_none());

    let err_req_param = IamCertConfLdapAddOrModifyReq {
        port: Some(293u16),
        ..conf_ldap_add_or_modify_req.clone()
    };
    assert!(IamCertLdapServ::add_cert_conf(&err_req_param, None, &funs, admin_ctx).await.is_err());

    let ldap_cert_conf_id = IamCertLdapServ::add_cert_conf(&conf_ldap_add_or_modify_req, None, &funs, admin_ctx).await.unwrap();
    let _ldap_cert_conf = IamCertLdapServ::get_cert_conf(&ldap_cert_conf_id, &funs, admin_ctx).await.unwrap();

    info!("【test ldap sync function】");
    let conf_id = IamCertServ::get_cert_conf_id_by_kind("Ldap", None, &funs).await.unwrap();
    assert_eq!(conf_id, ldap_cert_conf_id);

    let account_page = IamAccountServ::paginate_account_summary_aggs(
        &IamAccountFilterReq {
            basic: Default::default(),
            ..Default::default()
        },
        false,
        false,
        1,
        50,
        None,
        None,
        &funs,
        admin_ctx,
    )
    .await
    .unwrap();
    assert_eq!(account_page.total_size, 1);

    info!("【exec manual sync】");
    IamCertServ::third_integration_sync(
        Some(IamThirdIntegrationConfigDto {
            account_sync_from: IamCertExtKind::Ldap,
            account_sync_cron: None,
            account_way_to_add: WayToAdd::default(),
            account_way_to_delete: WayToDelete::default(),
        }),
        &funs,
        admin_ctx,
    )
    .await
    .unwrap();

    tokio::spawn(async {
        loop {
            let a = IamCertServ::get_third_intg_sync_status(&funs).await.unwrap();
            info!("get_third_intg_sync_status:{:?}", a);
            if a.is_some() {
                let a = a.unwrap();
                if a.total != 0 && (a.total == (a.failed + a.success) as usize) {
                    break;
                }
            }
        }
    });

    info!("【When a sync task is in progress, executing the sync method again will response an error】");
    assert!(IamCertServ::third_integration_sync(
        Some(IamThirdIntegrationConfigDto {
            account_sync_from: IamCertExtKind::Ldap,
            account_sync_cron: None,
            account_way_to_add: WayToAdd::default(),
            account_way_to_delete: WayToDelete::default(),
        }),
        &funs,
        admin_ctx,
    )
    .await
    .is_err());

    //wait for sync task done;
    sleep(Duration::from_millis(500)).await;

    let account_page = IamAccountServ::paginate_account_summary_aggs(
        &IamAccountFilterReq {
            basic: Default::default(),
            ..Default::default()
        },
        true,
        true,
        1,
        50,
        None,
        None,
        &funs,
        admin_ctx,
    )
    .await
    .unwrap();

    info!("【exec manual sync 2】");
    IamCertServ::third_integration_sync(
        Some(IamThirdIntegrationConfigDto {
            account_sync_from: IamCertExtKind::Ldap,
            account_sync_cron: None,
            account_way_to_add: WayToAdd::default(),
            account_way_to_delete: WayToDelete::default(),
        }),
        &funs,
        admin_ctx,
    )
    .await
    .unwrap();
    let account_ldap_cert: Vec<Option<&String>> = account_page.records.iter().map(|a| a.certs.get(&conf_ldap_add_or_modify_req.name)).filter(|o| o.is_some()).collect();

    assert_eq!(account_ldap_cert.len() as u64, ldap_account_num);
    assert_eq!(account_page.total_size, ldap_account_num + account_offset as u64);

    info!("【delete ldap conf and cert】");
    IamCertServ::delete_cert_and_conf_by_conf_id(&ldap_cert_conf_id, &funs, admin_ctx).await.unwrap();
    sleep(Duration::from_secs(1)).await;
    if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(admin_ctx).await.unwrap() {
        let mut is_finish = false;
        while is_finish {
            sleep(Duration::from_millis(100)).await;
            is_finish = TaskProcessor::check_status(&funs.conf::<IamConfig>().cache_key_async_task_status, task_id.parse::<i64>().unwrap(), funs.cache()).await.unwrap();
        }
    }
    let account_page = IamAccountServ::paginate_account_summary_aggs(
        &IamAccountFilterReq {
            basic: Default::default(),
            ..Default::default()
        },
        true,
        true,
        1,
        50,
        None,
        None,
        &funs,
        admin_ctx,
    )
    .await
    .unwrap();

    let account_ldap_cert: Vec<Option<&String>> = account_page.records.iter().map(|a| a.certs.get(&conf_ldap_add_or_modify_req.name)).filter(|o| o.is_some()).collect();
    assert!(account_ldap_cert.is_empty());

    let _ldap_cert_conf_id_2 = IamCertLdapServ::add_cert_conf(&conf_ldap_add_or_modify_req, None, &funs, admin_ctx).await.unwrap();

    info!("【exec manual sync 3】");

    let error_msg = IamCertLdapServ::iam_sync_ldap_user_to_iam(
        IamThirdIntegrationConfigDto {
            account_sync_from: IamCertExtKind::Ldap,
            account_sync_cron: None,
            account_way_to_add: WayToAdd::default(),
            account_way_to_delete: WayToDelete::default(),
        },
        &funs,
        admin_ctx,
    )
    .await
    .unwrap();

    error!("exec manual sync 3 error_msg: {}", error_msg);
    let account_page = IamAccountServ::paginate_account_summary_aggs(
        &IamAccountFilterReq {
            basic: Default::default(),
            ..Default::default()
        },
        true,
        true,
        1,
        50,
        None,
        None,
        &funs,
        admin_ctx,
    )
    .await
    .unwrap();
    println!("================={:?}", account_page.records);
    assert_eq!(account_page.total_size, ldap_account_num * 2 + account_offset as u64);

    funs.commit().await.unwrap();
}
