use spacegate_shell::hyper::HeaderMap;

#[derive(Clone)]
pub struct AuditLogParam {
    pub request_path: String,
    pub request_method: String,
    pub request_headers: HeaderMap,
    pub request_scheme: String,
    pub request_ip: String,
}
