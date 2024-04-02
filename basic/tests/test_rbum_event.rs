use std::sync::atomic::{AtomicUsize, Ordering};

use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::TardisFuns;

use bios_basic::rbum::dto::rbum_set_dto::RbumSetAddReq;
use bios_basic::rbum::helper::rbum_event_helper;
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_set_serv::RbumSetServ;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub async fn test() -> TardisResult<()> {
    let funs = TardisFuns::inst_with_db_conn("".to_string(), None);
    let ctx = TardisContext {
        own_paths: "".to_string(),
        owner: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        ..Default::default()
    };
    info!("【test_rbum_event】 : receive events");
    rbum_event_helper::receive(
        |(_, msg)| async move {
            let message = rbum_event_helper::parse_message(msg)?;
            assert_eq!(message.table_name, "rbum_set");
            assert_eq!(message.operate, "c");
            COUNTER.fetch_add(1, Ordering::SeqCst);
            Ok(())
        },
        &funs,
    )
    .await?;

    RbumSetServ::add_rbum(
        &mut RbumSetAddReq {
            code: TrimString("test_rbum_set_code".to_string()),
            kind: TrimString("".to_string()),
            name: TrimString(" 测试集合 ".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::L2),
            ext: None,
            disabled: None,
        },
        &funs,
        &ctx,
    )
    .await?;

    if let Some(notify_events) = rbum_event_helper::get_notify_event_with_ctx(&ctx).await? {
        rbum_event_helper::try_notifies(notify_events, &funs, &ctx).await?;
    }

    loop {
        if COUNTER.load(Ordering::SeqCst) > 0 {
            break;
        }
    }
    Ok(())
}
