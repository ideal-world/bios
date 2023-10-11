pub const DOMAIN_CODE: &str = "spi-conf";
pub const DOMAIN_CODE_NACOS: &str = "spi-conf-nacos";
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
            CONLICT_AK:                 409 = "conlict-username";
            EXCEED_MAX_RETRY_TIMES:           409 = "exceed-max-retry-times";
            VALID_ERROR:                401 = "valid-error";
            CACHE_ERROR:                500 = "cache-error";
        }
    }
}

/// spi-conf cert kind
pub const SPI_CONF_CERT_KIND: &str = "spi-conf";

// for generate string
pub const CHARSET_AK: &[u8] = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ~!@#$%^&*()_+".as_bytes();
pub const CHARSET_SK: &[u8] = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_".as_bytes();
