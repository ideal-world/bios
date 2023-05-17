use bios_basic::spi::spi_funs::SpiBsInstExtractor;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    db::{reldb_client::TardisRelDBClient, sea_orm::Value},
    web::web_resp::TardisPage,
    TardisFunsInst,
};

use crate::{
    dto::{conf_config_dto::*, conf_namespace_dto::*},
    serv::pg::conf_pg_initializer,
};

pub async fn create_namespace(attribute: &mut NamespaceAttribute, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    let mut params = vec![Value::from(&attribute.namespace), Value::from(&attribute.namespace_show_name)];
    params.extend(attribute.namespace_desc.as_ref().map(Value::from));
    let bs_inst = funs.bs(ctx).await?.inst::<TardisRelDBClient>();
    let (mut conn, table_name) = conf_pg_initializer::init_table_and_conn_namespace(bs_inst, ctx, true).await?;
    conn.begin().await?;
    conn.execute_one(
        &format!(
            r#"INSERT INTO {table_name} 
    (id, show_name{})
VALUES
    ($1, $2{})
	"#,
            if attribute.namespace_desc.is_some() { ", desc" } else { "" },
            if attribute.namespace_desc.is_some() { ", $3" } else { "" },
        ),
        params,
    )
    .await?;
    conn.commit().await?;
    Ok(())
}
