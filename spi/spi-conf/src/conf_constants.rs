pub const DOMAIN_CODE: &str = "spi-conf";
pub mod error {
    macro_rules! def_error_code {
        (
            $domain_code: literal {
                $($name:ident: $code:literal = $description:literal;)*
            }) => {
            $ (
                pub const $name: &str = concat!($code, "-", $domain_code, "-", $description);
            ) *
        };
    }
    def_error_code! {
        "spi-conf" {
            NAMESPACE_DEFAULT_CANNOT_DELETE: 400 = "default-namespace-cannot-be-deleted";
            INVALID_UUID:               400 = "invalid-uuid";
            CONF_NOTFOUND:              404 = "conf-not-exist";
            NAMESPACE_NOTFOUND:         404 = "namespace-not-exist";
        }
    }
}
