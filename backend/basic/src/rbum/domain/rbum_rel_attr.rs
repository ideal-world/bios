use tardis::chrono::{self, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// Resource relationship attribute condition model
///
/// 资源关联属性条件模型
///
/// This model is used to further qualify the conditions under which the relationship is established.
///
/// 该模型用于进一步限定建立关联关系的条件。
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "rbum_rel_attr")]
pub struct Model {
    /// Relationship attribute id
    ///
    /// 关联属性id
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Condition qualifier
    ///
    /// 条件限定符
    ///
    /// if true, it means the limitation of the relationship source,
    /// otherwise it is the limitation of the relationship target resource.
    ///
    /// 如果为true，表示关联来源方的限定，否则为关联目标方资源的限定。
    pub is_from: bool,
    /// Relationship attribute name
    ///
    /// 关联属性名称
    ///
    /// When ``rel_rbum_kind_attr_id`` exists, use the corresponding [`crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp::name`], otherwise this field is not empty.
    ///
    /// 当 ``rel_rbum_kind_attr_id`` 存在时使用其对应的 [`crate::rbum::dto::rbum_kind_attr_dto::RbumKindAttrDetailResp::name`]，否则此字段不为空。
    #[index]
    pub name: String,
    /// Relationship attribute value
    ///
    /// 关联属性值
    pub value: String,
    /// Whether to only record
    ///
    /// 是否仅记录
    ///
    /// If true, this condition is only used for records and does not participate in the judgment of whether the relationship is established.
    ///
    /// 如果为true，该条件仅用于记录，不参与判断关联关系是否建立。
    pub record_only: bool,
    /// Associated [resource kind attribute](crate::rbum::domain::rbum_kind_attr::Model) id
    ///
    /// 关联的[资源类型属性](crate::rbum::domain::rbum_kind_attr::Model) id
    pub rel_rbum_kind_attr_id: String,
    /// Associated [relationship](crate::rbum::domain::rbum_rel::Model) id
    ///
    /// 关联的[资源关联](crate::rbum::domain::rbum_rel::Model) id
    #[index]
    pub rel_rbum_rel_id: String,

    #[fill_ctx(fill = "own_paths")]
    pub own_paths: String,
    #[fill_ctx]
    pub owner: String,
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub create_time: chrono::DateTime<Utc>,
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub update_time: chrono::DateTime<Utc>,
    #[fill_ctx]
    pub create_by: String,
    #[fill_ctx(insert_only = false)]
    pub update_by: String,
}
