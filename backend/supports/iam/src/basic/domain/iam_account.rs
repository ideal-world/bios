use tardis::chrono::{self, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "iam_account")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub icon: String,
    /// [data type Kind](crate::iam_enumeration::IamAccountStatusKind)
    pub status: i16,
    /// Marking of temporary status
    ///
    /// 临时状态的标记
    pub temporary: bool,
    /// [data type Kind](crate::iam_enumeration::IamAccountLockStateKind)
    pub lock_status: i16,
    /// Expanded fields with index
    ///
    /// 索引扩展字段 idx 1-3
    #[index]
    pub ext1_idx: String,
    #[index]
    pub ext2_idx: String,
    #[index]
    pub ext3_idx: String,
    /// Expanded fields
    ///
    /// 普通扩展字段 4-9
    pub ext4: String,
    pub ext5: String,
    pub ext6: String,
    pub ext7: String,

    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub effective_time: chrono::DateTime<Utc>,

    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub logout_time: chrono::DateTime<Utc>,
    /// [data type Kind](crate::iam_enumeration::IamAccountLogoutTypeKind)
    pub logout_type: String,

    /// Nature of labor
    ///
    /// 用工性质
    pub labor_type: String,

    /// ID card number
    ///
    /// 身份证号码
    pub id_card_no: String,

    /// Employee Code
    ///
    /// 工号
    pub employee_code: String,

    #[fill_ctx(fill = "own_paths")]
    pub own_paths: String,
}
