use tardis::derive_more::Display;
#[cfg(feature = "default")]
use tardis::web::poem_openapi;

#[derive(Display, Debug)]
#[cfg_attr(feature = "default", derive(poem_openapi::Tags))]
pub enum ApiTag {
    #[oai(rename = "Common Console")]
    Common,
    #[oai(rename = "Tenant Console")]
    Tenant,
    #[oai(rename = "App Console")]
    App,
    #[oai(rename = "System Console")]
    System,
    #[oai(rename = "Passport Console")]
    Passport,
    #[oai(rename = "Interface Console")]
    Interface,
}
