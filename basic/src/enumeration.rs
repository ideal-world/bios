use derive_more::Display;
use tardis::db::sea_orm::strum::EnumString;
use tardis::web::poem_openapi::Tags;

#[derive(Tags, Display, EnumString, Debug)]
pub enum Components {
    /// IAM Component
    #[oai(rename = "IAM")]
    #[display(fmt = "iam")]
    Iam,
}
