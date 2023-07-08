use tardis::web::poem_openapi;

#[derive(Debug, poem_openapi::Enum)]
#[oai(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReachChannelKind {
    Sms,
    Email,
    Inbox,
    Wechat,
    DingTalk,
    Push,
    WebHook,
}

#[derive(Debug, poem_openapi::Enum)]
#[oai(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReachReceiveKind {
    Account,
    Role,
    App,
    Tenant,
}
#[derive(Debug, poem_openapi::Enum)]
#[oai(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReachStatusKind {
    Draft,
    /// 定时消息未发送时的状态
    Pending,
    /// 非 [ReachChannelKind::Inbox] 类型的发送中的状态
    Sending,
    SendSuccess,
    /// 多用于站内信
    AllDelevery,
    /// 非 [ReachChannelKind::Inbox] 类型的失败的状态
    Fail,
}
#[derive(Debug, poem_openapi::Enum)]
pub enum ReachDndStrategyKind {
    Ignore,
    Delay,
}

#[derive(Debug, poem_openapi::Enum)]
#[oai(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReachLevelKind {
    Urgent,
    High,
    Normal,
    Low,
}


#[derive(Debug, poem_openapi::Enum)]
#[oai(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReachTimeoutStrategyKind {
    Ignore,
    RetryOnce,
}

#[derive(Debug, poem_openapi::Enum)]
#[oai(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReachTemplateKind {
    Vcode,
    Notice,
    Promote,
}