use tardis::basic::result::TardisResult;
use tardis::TardisFuns;

pub async fn init() -> TardisResult<()> {
    TardisFuns::reldb().create_table_from_entity(bios_com_iam::domain::rbum_kind::Entity).await?;
    TardisFuns::web_server().add_module("iam", bios_com_iam::controller::processor::TodoApi).start().await
}
