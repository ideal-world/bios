use tardis::chrono::{self, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

/// Certificate configuration model
///
/// 凭证配置模型
///
/// This is a general certificate configuration model that describes various types of certificate configurations.
/// It can handle database connections, web system authentication, host ssh, etc.
///
/// 这是一个通用的凭证配置模型，用于描述各种类型的凭证配置。可处理包含数据库连接、web系统认证、主机ssh谁等。
///
/// Certificate configuration is bound to a subject that can be a resource domain [`Self::rel_rbum_domain_id`] or a resource item [`Self::rel_rbum_item_id`].
/// It is required that the same binding subject has the same certificate configuration type [`Self::kind`] and certificate configuration supplier [`Self::supplier`] unique.
///
/// 凭证配置绑定的主体可以是某一资源域[`Self::rel_rbum_domain_id`]或资源项[`Self::rel_rbum_item_id`]。要求同一绑定主体下同一凭证配置类型[`Self::kind`]及凭证配置供应商[`Self::supplier`]唯一。
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "rbum_cert_conf")]
pub struct Model {
    /// Certificate configuration ID
    ///
    /// 凭证配置ID
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Certificate configuration type
    ///
    /// 凭证配置类型
    ///
    /// Used for the classification of certificates, such as: ldap, userpwd, token, oauth2, etc.
    ///
    /// 用于凭证的分类，比如：ldap、userpwd、token、oauth2等。
    #[tardis_entity(custom_len = "127")]
    #[index(index_id = "id_2", unique)]
    pub kind: String,
    /// Certificate configuration supplier
    ///
    /// 凭证配置供应商
    ///
    /// One type of certificate can have multiple suppliers.
    /// For example, the certificate of type oauth2 can be further refined into WeChat oauth2, QQ oauth2, Weibo oauth2, etc.
    ///
    /// 一种凭证类型可以有多个供应商。比如 oauth2 类型的凭证，可以进一步细化成 微信oauth2、QQ oauth2、微博 oauth2 等。
    #[tardis_entity(custom_len = "127")]
    #[index(index_id = "id_2", unique)]
    pub supplier: String,
    /// Certificate configuration name
    ///
    /// 凭证配置名称
    pub name: String,
    /// Certificate configuration description
    ///
    /// 凭证配置描述
    pub note: String,
    /// Certificate configuration ak rule(verification regular expression)
    ///
    /// 凭证配置ak规则（校验正则）
    ///
    /// # What is ``AK``
    ///
    /// For username and password type credentials, ``AK`` refers to the username;
    /// for token type credentials, ``AK`` is empty;
    /// for oauth2 type (authorization code) credentials, ``AK`` refers to the client_id value,
    /// for mobile phone verification code type credentials, ``AK`` refers to the mobile phone number, etc.
    ///
    /// # 什么是``AK``
    ///
    /// 对于用户名密码类型的凭证，``AK``指的是用户名；对于token类型的凭证，``AK``为空；对于oauth2类型（授权码模式）的凭证，``AK``指的是client_id值；对于手机号验证码类型的凭证，``AK``指的是手机号等。
    pub ak_rule: String,
    /// Certificate configuration ak rule description
    ///
    /// 凭证配置ak规则的描述
    pub ak_note: String,
    /// Certificate configuration sk rule(verification regular expression)
    ///
    /// 凭证配置sk规则（校验正则）
    ///
    /// # What is ``SK``
    ///
    /// For username and password type credentials, ``SK`` refers to the password;
    /// for token type credentials, ``SK`` refers to the token value;
    /// for oauth2 type (authorization code) credentials, ``SK`` refers to the access token value,
    /// for mobile phone verification code type credentials, ``SK`` is empty, etc.
    ///
    /// # 什么是``SK``
    ///
    /// 对于用户名密码类型的凭证，``SK``指的是用户名；对于token类型的凭证，``SK``指的是token值；对于oauth2类型（授权码模式）的凭证，``SK``为access token值；对于手机号验证码类型的凭证，``SK``为空等。
    pub sk_rule: String,
    /// Certificate configuration sk rule description
    ///
    /// 凭证配置sk规则的描述
    pub sk_note: String,
    /// Certificate configuration extension information
    ///
    /// 凭证配置扩展信息
    ///
    /// Such as database connection pool configuration.
    ///
    /// 比如数据库连接池配置。
    #[tardis_entity(custom_type = "text")]
    pub ext: String,
    /// Whether sk is required
    ///
    /// 是否需要指定sk
    ///
    /// Some credentials' sk is generated by the system, such as token type credentials, the token value can be generated by the system.
    ///
    /// 有些凭证的sk由系统生成，比如token类型的凭证，token值可以由系统生成。
    pub sk_need: bool,
    /// Whether sk is dynamic
    ///
    /// 是否动态sk
    ///
    /// Mainly used for ``verification code`` type sk, such as mobile phone verification code, email verification code, etc., a new sk will be generated each time you log in.
    ///
    /// 多用于``验证码``类型的sk，比如手机验证码、邮箱验证码等，每次登录都会生成一个新的sk。
    ///
    /// NOTE: Credential processing provides a relatively general verification code processing logic. See [`crate::rbum::serv::rbum_cert_serv::RbumCertServ`]
    ///
    /// NOTE: 凭证处理的提供了一个相对通用的验证码处理逻辑。见 [`crate::rbum::serv::rbum_cert_serv::RbumCertServ`]
    pub sk_dynamic: bool,
    /// Whether sk is encrypted
    ///
    /// 是否加密sk
    ///
    /// Enabling encryption will use sha512 with salt to save. See [`crate::rbum::serv::rbum_cert_serv::RbumCertServ::encrypt_sk`]
    ///
    /// 启用加密会使用sha512加盐保存。见 [`crate::rbum::serv::rbum_cert_serv::RbumCertServ::encrypt_sk`]
    pub sk_encrypted: bool,
    /// Whether sk can be repeated (same as the ak before modification)
    ///
    /// 修改sk是否可以重复（与修改前的ak相同）
    ///
    /// When the password expires, a new different password must be set to improve security.
    ///
    /// 可以设置在密码过期后，必须设置新的不同的密码，以提升安全性。
    pub repeatable: bool,
    /// Whether it is a basic authentication
    ///
    /// 是否为基础认证
    ///
    /// There can only be at most one base certification for the same `rel_rbum_item_id` ,
    /// If true, the sk of this record will be the public sk of the same `rel_rbum_item_id` ,
    /// supports a login method like ak of different cert configuration in the same `rel_rbum_item_id` + sk of this record.
    /// For example, the password can be used as the basic sk, so that the login methods of mobile phone verification code, username password, and mobile phone + password can be realized.
    ///
    /// 同一个`rel_rbum_item_id`下最多只能有一个基础认证，如果为true，则该记录的sk将为同一个`rel_rbum_item_id`下的公共sk，支持同一个`rel_rbum_item_id`下不同凭证配置的ak + 该记录的sk的登录方式。
    /// 比如可以将密码作为基础sk，这样可以实现手机号验证码、用户名密码以及手机号+密码的登录方式。
    pub is_basic: bool,
    /// Support reset the cert configuration type(corresponding to the ``code`` value) of the basic sk
    ///
    /// 支持重置基础sk的凭证配置类型（对应`code`值）
    ///
    ///
    /// Multiple values are separated by commas.
    ///
    /// 多个值用逗号分隔。
    pub rest_by_kinds: String,
    /// The expiration time of the sk
    ///
    /// sk的过期时间
    pub expire_sec: i64,
    /// sk被锁定的最大错误次数，超过此次数将被锁定
    ///
    /// The maximum number of errors that sk can be locked, and it will be locked if it exceeds this number
    ///
    /// WARING: If an object (such as a user) is bound to multiple credential configurations, and these credential configurations have different sk_lock_err_times, it may be based on the maximum.
    ///
    /// WARING: 如果某个对象（比如用户）绑定了多个凭证配置，且这些凭证配置了不同的sk_lock_err_times，那有可能会以最大的为准。
    pub sk_lock_err_times: i16,
    /// sk lock duration
    ///
    /// sk被锁定的持续时间
    pub sk_lock_duration_sec: i32,
    /// sk validation error count cycle
    ///
    /// sk校验错误次数的周期
    ///
    /// If the time of the last error is greater than this cycle, the count is reset.
    ///
    /// 如果上次发生错误的时间大于此周期，则重新计数。
    pub sk_lock_cycle_sec: i32,
    /// The number of certificates in effect at the same time
    ///
    /// 同时有效的凭证数量
    ///
    /// If only single terminal login is allowed, you can configure a credential configuration ``coexist_num = 1``, and ensure that all terminals use this configuration when logging in.
    /// If an android, ios, and two web terminals can log in at the same time,
    /// you can configure three credential configurations: ``code = 'cert_android' & coexist_num = 1``, ``code = 'cert_ios' & coexist_num = 1``, ``code = 'cert_web' & coexist_num = 2``,
    /// and ensure that android login uses the configuration ``code = 'cert_android'``, ios login uses the configuration ``code = 'cert_ios'``, and web login uses the configuration ``code = 'cert_web'``.
    ///
    /// 如果只能单终端登录，可以配置一条凭证配置 ``coexist_num = 1`` ，并确保所有终端登录时都使用此配置。
    /// 如果可以同时登录一个android、ios、两个web终端，可以配置三条凭证配置：``name = 'cert_android' & coexist_num = 1`` 、 ``name = 'cert_ios' & coexist_num = 1 ``、 ``name = 'cert_web' & coexist_num = 2``，
    /// 并确保android登录使用``name = 'cert_android'``的配置，ios登录使用``name = 'cert_ios'``的配置，web登录使用``name = 'cert_web'``的配置。
    pub coexist_num: i16,
    /// Specifies the connection address
    ///
    /// 指定连接地址
    ///
    /// For example, the authentication address of oauth2, the database connection address, etc.
    ///
    /// 比如oauth2的认证地址、数据库连接地址等。
    pub conn_uri: String,
    /// Credential configuration status
    ///
    /// 凭证配置的状态
    ///
    /// see[`crate::rbum::rbum_enumeration::RbumCertConfStatusKind`]
    pub status: i16,
    /// Associated [resource domain](crate::rbum::domain::rbum_domain::Model) id
    ///
    /// 关联的[资源域](crate::rbum::domain::rbum_domain::Model) id
    #[index(index_id = "id_2", unique)]
    pub rel_rbum_domain_id: String,
    /// Associated [resource](crate::rbum::domain::rbum_item::Model) id
    ///
    /// 关联的[资源](crate::rbum::domain::rbum_item::Model) id
    #[index(index_id = "id_2", unique)]
    pub rel_rbum_item_id: String,

    #[index()]
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
