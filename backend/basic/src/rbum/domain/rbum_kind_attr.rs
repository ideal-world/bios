use tardis::chrono::{self, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::sea_query::{ColumnDef, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// Resource kind extended attribute definition model
///
/// General logic for dynamic request processing:
///
/// 1. dynamic values take precedence over static values
/// 2. supports calling http to get data with GET request
/// 3. request url supports attribute variable substitution, format is: `{attribute name}` .
/// 4. if no attribute variable substitution exists and secret = false, the url is called directly and the corresponding value is returned,
/// 5. if attribute variable substitution exists, then：
///  1） extract all attribute variables to be replaced
///  2） monitor changes of these attributes
///  3） substitute attribute variables with values into the url
///  4） if no longer an attribute variable substitution in the url and secret = false, call the url and return the corresponding value
/// 6. before the resource object is saved, if secret = true and an attribute variable substitution in the url, call the url and return the corresponding value
///
/// For security reasons, step 6 must be done by the server side.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "rbum_kind_attr")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,

    #[index(index_id = "id")]
    pub name: String,
    #[index(index_id = "id")]
    pub module: String,
    pub label: String,
    pub note: String,
    pub sort: i64,
    pub main_column: bool,
    pub position: bool,
    pub capacity: bool,
    pub overload: bool,
    pub hide: bool,
    /// When secret = true, the attribute information is not returned to the frontend(except in configuration)
    pub secret: bool,
    /// Display condition, json format: `{<attribute name>:<attribute value>}`, currently only supports `and` operations
    pub show_by_conds: String,
    /// Whether indexing is needed
    pub idx: bool,
    /// Associated [resource kind](crate::rbum::rbum_enumeration::RbumDataTypeKind)
    pub data_type: String,
    /// Associated [resource kind](crate::rbum::rbum_enumeration::RbumWidgetTypeKind)
    pub widget_type: String,
    #[tardis_entity(default_value = 1)]
    pub widget_columns: i16,
    pub default_value: String,
    /// Dynamic default value
    /// the return format is the same as `default_value`
    /// or `json` when `data_type` = `Json` and `widget_type` = `Control`
    /// or `array` when `data_type` = `Array` and `widget_type` = `Group`
    pub dyn_default_value: String,
    /// Fixed option, json array formatted as `[{name:<display name>:value:<corresponding value>}]`
    pub options: String,
    /// Dynamic options
    /// the return format is the same as `options`
    pub dyn_options: String,
    pub required: bool,
    pub min_length: i32,
    pub max_length: i32,
    /// Used to implement multi-level attributes, default is empty
    pub parent_attr_name: String,
    /// Custom behavior attributes \
    /// E.g. user selection function, role selection function, etc.
    /// Custom behavior needs to be bound to the corresponding function code.
    pub action: String,
    pub ext: String,
    /// Associated [resource kind](crate::rbum::domain::rbum_kind::Model) id
    #[index(index_id = "id")]
    pub rel_rbum_kind_id: String,

    pub scope_level: i16,

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
