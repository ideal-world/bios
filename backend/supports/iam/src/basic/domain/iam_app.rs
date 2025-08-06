use tardis::db::sea_orm;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

use crate::basic::dto::iam_app_dto::IamAppKind;
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "iam_app")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub description: Option<String>,
    pub icon: String,
    pub sort: i64,
    // 联系号码
    pub contact_phone: String,

    #[tardis_entity(custom_type = "String")]
    pub kind: IamAppKind,

    #[fill_ctx(fill = "own_paths")]
    pub own_paths: String,
}
