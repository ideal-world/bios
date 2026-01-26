//! LDAP System Result Builder
//!
//! 负责组装LDAP根查询和Schema查询的响应结果

use ldap3_proto::simple::*;

use crate::iam_config::IamLdapConfig;
use crate::integration::ldap::ldap_parser::{is_root_dse_query, is_subschema_query, LdapBaseDnLevel, LdapQueryType, LdapSearchQuery};

/// 构建LDAP根查询响应
pub fn build_root_dse_search_response(req: &SearchRequest, query: &LdapSearchQuery, config: &IamLdapConfig) -> Vec<LdapMsg> {
    // 根DSE只支持base scope
    if req.scope != LdapSearchScope::Base {
        return vec![req.gen_error(LdapResultCode::ProtocolError, "Root DSE only supports base scope".to_string())];
    }

    let mut results = Vec::new();
    let root_dse_attributes = build_root_dse_attributes(config, query);
    results.push(req.gen_result_entry(LdapSearchResultEntry {
        dn: "".to_string(),
        attributes: root_dse_attributes,
    }));
    results.push(req.gen_success());
    results
}

/// 构建LDAP Schema查询响应
pub fn build_subschema_search_response(req: &SearchRequest, query: &LdapSearchQuery, config: &IamLdapConfig) -> Vec<LdapMsg> {
    // Schema查询只支持base scope
    if req.scope != LdapSearchScope::Base {
        return vec![req.gen_error(LdapResultCode::ProtocolError, "Schema query only supports base scope".to_string())];
    }
    let mut results = Vec::new();
    let schema_attributes = build_subschema_attributes(config, query);
    results.push(req.gen_result_entry(LdapSearchResultEntry {
        dn: config.schema_dn.clone(),
        attributes: schema_attributes,
    }));
    results.push(req.gen_success());
    results
}

/// 根据请求的属性列表过滤属性
/// 根据LDAP协议：
/// - 如果请求列表为空或包含"*"，返回所有用户属性
/// - 如果请求了"+*"，返回所有操作属性（当前实现不返回操作属性）
/// - 否则只返回请求的属性
fn filter_attributes_by_request(all_attributes: &[LdapPartialAttribute], requested_attrs: &[String]) -> Vec<LdapPartialAttribute> {
    // 如果请求列表为空或包含"*"，返回所有属性
    if requested_attrs.is_empty() || requested_attrs.iter().any(|attr| attr == "*") {
        return all_attributes.to_vec();
    }

    // 如果请求了"+*"，返回所有属性（操作属性当前未实现）
    if requested_attrs.iter().any(|attr| attr == "+*") {
        return all_attributes.to_vec();
    }

    // 只返回请求的属性（不区分大小写）
    let requested_lower: Vec<String> = requested_attrs.iter().map(|a| a.to_lowercase()).collect();
    all_attributes.iter().filter(|attr| requested_lower.contains(&attr.atype.to_lowercase())).cloned().collect()
}

/// 构建 RootDSE 属性
/// RootDSE (Root Directory Service Entry) 是 LDAP 服务器的根入口点
/// 它包含服务器的能力信息和配置信息
fn build_root_dse_attributes(config: &IamLdapConfig, query: &LdapSearchQuery) -> Vec<LdapPartialAttribute> {
    // 判断是否为 (!(objectClass=*)) 的情况
    if let LdapQueryType::Not { filter } = &query.query_type {
        if matches!(filter.as_ref(), LdapQueryType::Present { attribute } if attribute == "objectClass") {
            return vec![];
        }
    }
    // 构建所有可用的 RootDSE 属性
    let all_attributes = vec![
        // namingContexts: 命名上下文（base DN）
        LdapPartialAttribute {
            atype: "namingContexts".to_string(),
            vals: vec![config.base_dn.clone().into()],
        },
        // subschemaSubentry: Schema 子条目位置（Apache Directory Studio 需要此属性）
        LdapPartialAttribute {
            atype: "subschemaSubentry".to_string(),
            vals: vec![config.schema_dn.clone().into()],
        },
        // supportedLDAPVersion: 支持的 LDAP 版本
        LdapPartialAttribute {
            atype: "supportedLDAPVersion".to_string(),
            vals: vec!["3".to_string().into()],
        },
        // supportedSASLMechanisms: 支持的 SASL 机制
        LdapPartialAttribute {
            atype: "supportedSASLMechanisms".to_string(),
            vals: vec!["PLAIN".to_string().into()],
        },
        // vendorName: 供应商名称
        LdapPartialAttribute {
            atype: "vendorName".to_string(),
            vals: vec!["BIOS".to_string().into()],
        },
        // vendorVersion: 供应商版本
        LdapPartialAttribute {
            atype: "vendorVersion".to_string(),
            vals: vec!["1.0".to_string().into()],
        },
    ];

    // 根据请求的属性列表过滤属性
    filter_attributes_by_request(&all_attributes, &query.attributes)
}

/// 构建 Schema (Subschema) 属性
/// Schema 查询用于返回 LDAP 服务器的 schema 定义信息
/// 这是 Apache Directory Studio 等客户端在连接时需要的标准查询
fn build_subschema_attributes(_config: &IamLdapConfig, query: &LdapSearchQuery) -> Vec<LdapPartialAttribute> {
    // 判断是否为 (!(objectClass=subschema)) 的情况
    if let LdapQueryType::Not { filter } = &query.query_type {
        if matches!(filter.as_ref(), LdapQueryType::Equality { attribute, value } if attribute == "objectClass" && value == "subschema") {
            return vec![];
        }
    }
    // 构建所有可用的 Schema 属性
    let mut all_attributes = vec![
        // objectClass: 标识这是一个 subschema 条目
        LdapPartialAttribute {
            atype: "objectClass".to_string(),
            vals: vec!["subschema".to_string().into(), "top".to_string().into()],
        },
    ];

    // objectClasses: 对象类定义（我们支持的对象类）
    let object_classes = vec![
        "( 2.5.6.6 NAME 'person' SUP top STRUCTURAL MUST ( sn $ cn ) MAY ( userPassword $ telephoneNumber $ seeAlso $ description ) )",
        "( 2.5.6.7 NAME 'organizationalPerson' SUP person STRUCTURAL MAY ( title $ x121Address $ registeredAddress $ destinationIndicator $ preferredDeliveryMethod $ telexNumber $ teletexTerminalIdentifier $ telephoneNumber $ internationaliSDNNumber $ facsimileTelephoneNumber $ street $ postOfficeBox $ postalCode $ postalAddress $ physicalDeliveryOfficeName $ ou $ st $ l ) )",
        "( 2.5.6.8 NAME 'organizationalUnit' SUP top STRUCTURAL MUST ou MAY ( businessCategory $ description $ destinationIndicator $ facsimileTelephoneNumber $ internationaliSDNNumber $ l $ physicalDeliveryOfficeName $ postOfficeBox $ postalAddress $ postalCode $ preferredDeliveryMethod $ registeredAddress $ searchGuide $ seeAlso $ st $ street $ telephoneNumber $ teletexTerminalIdentifier $ telexNumber $ userPassword $ x121Address ) )",
        "( 1.3.6.1.4.1.1466.344 NAME 'dcObject' SUP top AUXILIARY MUST dc )",
        "( 2.5.6.0 NAME 'top' ABSTRACT MUST objectClass )",
        "( 1.3.6.1.1.3.1 NAME 'uidObject' SUP top AUXILIARY MAY uid )",
        "( 2.16.840.1.113730.3.2.2 NAME 'inetOrgPerson' SUP organizationalPerson STRUCTURAL MAY ( audio $ businessCategory $ carLicense $ departmentNumber $ displayName $ employeeNumber $ employeeType $ givenName $ homePhone $ homePostalAddress $ initials $ jpegPhoto $ labeledURI $ mail $ manager $ mobile $ o $ pager $ photo $ roomNumber $ secretary $ uid $ userCertificate $ x500uniqueIdentifier $ preferredLanguage $ userSMIMECertificate $ userPKCS12 ) )",
    ];
    all_attributes.push(LdapPartialAttribute {
        atype: "objectClasses".to_string(),
        vals: object_classes.into_iter().map(|s| s.into()).collect(),
    });

    // attributeTypes: 属性类型定义（我们支持的属性）
    let attribute_types = vec![
        "( 2.5.4.3 NAME ( 'cn' 'commonName' ) SUP name )",
        "( 2.5.4.4 NAME ( 'sn' 'surname' ) SUP name )",
        "( 2.5.4.5 NAME 'serialNumber' )",
        "( 2.5.4.6 NAME 'c' SUP name SINGLE-VALUE )",
        "( 2.5.4.7 NAME ( 'l' 'localityName' ) SUP name )",
        "( 2.5.4.8 NAME ( 'st' 'stateOrProvinceName' ) SUP name )",
        "( 2.5.4.9 NAME ( 'street' 'streetAddress' ) SUP name )",
        "( 2.5.4.10 NAME 'o' SUP name )",
        "( 2.5.4.11 NAME ( 'ou' 'organizationalUnitName' ) SUP name )",
        "( 2.5.4.12 NAME 'title' )",
        "( 2.5.4.13 NAME 'description' )",
        "( 2.5.4.15 NAME 'businessCategory' )",
        "( 2.5.4.20 NAME 'telephoneNumber' )",
        "( 2.5.4.25 NAME 'postalCode' )",
        "( 2.5.4.26 NAME 'postalAddress' )",
        "( 2.5.4.31 NAME 'member' )",
        "( 2.5.4.41 NAME 'name' )",
        "( 2.5.4.42 NAME ( 'givenName' 'gn' ) SUP name )",
        "( 2.5.4.43 NAME 'initials' SUP name )",
        "( 2.5.4.44 NAME 'generationQualifier' SUP name )",
        "( 2.5.4.46 NAME 'dnQualifier' )",
        "( 2.5.4.51 NAME 'houseIdentifier' )",
        "( 0.9.2342.19200300.100.1.1 NAME ( 'uid' 'userid' ) )",
        "( 0.9.2342.19200300.100.1.3 NAME ( 'mail' 'rfc822Mailbox' ) EQUALITY caseIgnoreIA5Match SUBSTR caseIgnoreIA5SubstringsMatch SYNTAX 1.3.6.1.4.1.1466.115.121.1.26{256} )",
        "( 0.9.2342.19200300.100.1.25 NAME 'dc' EQUALITY caseIgnoreMatch SUBSTR caseIgnoreSubstringsMatch SYNTAX 1.3.6.1.4.1.1466.115.121.1.44 )",
        "( 1.2.840.113556.1.4.221 NAME 'sAMAccountName' SYNTAX 1.3.6.1.4.1.1466.115.121.1.15{256} )",
        "( 2.16.840.1.113730.3.1.241 NAME 'displayName' )",
        "( 2.16.840.1.113730.3.1.3 NAME 'employeeNumber' )",
        "( 2.16.840.1.113730.3.1.4 NAME 'employeeType' )",
        "( 0.9.2342.19200300.100.1.60 NAME 'jpegPhoto' SYNTAX 1.3.6.1.4.1.1466.115.121.1.28 )",
        "( 2.16.840.1.113730.3.1.39 NAME 'mobile' EQUALITY telephoneNumberMatch SUBSTR telephoneNumberSubstringsMatch SYNTAX 1.3.6.1.4.1.1466.115.121.1.50 )",
    ];
    all_attributes.push(LdapPartialAttribute {
        atype: "attributeTypes".to_string(),
        vals: attribute_types.into_iter().map(|s| s.into()).collect(),
    });

    // ldapSyntaxes: LDAP 语法定义（常用的语法）
    let ldap_syntaxes = vec![
        "( 1.3.6.1.4.1.1466.115.121.1.3 DESC 'Attribute Type Description' )",
        "( 1.3.6.1.4.1.1466.115.121.1.5 DESC 'Binary' )",
        "( 1.3.6.1.4.1.1466.115.121.1.6 DESC 'Bit String' )",
        "( 1.3.6.1.4.1.1466.115.121.1.7 DESC 'Boolean' )",
        "( 1.3.6.1.4.1.1466.115.121.1.8 DESC 'Certificate' )",
        "( 1.3.6.1.4.1.1466.115.121.1.9 DESC 'Certificate List' )",
        "( 1.3.6.1.4.1.1466.115.121.1.10 DESC 'Certificate Pair' )",
        "( 1.3.6.1.4.1.1466.115.121.1.11 DESC 'Country String' )",
        "( 1.3.6.1.4.1.1466.115.121.1.12 DESC 'Distinguished Name' )",
        "( 1.3.6.1.4.1.1466.115.121.1.13 DESC 'Data Quality' )",
        "( 1.3.6.1.4.1.1466.115.121.1.14 DESC 'Delivery Method' )",
        "( 1.3.6.1.4.1.1466.115.121.1.15 DESC 'Directory String' )",
        "( 1.3.6.1.4.1.1466.115.121.1.16 DESC 'DIT Content Rule Description' )",
        "( 1.3.6.1.4.1.1466.115.121.1.17 DESC 'DIT Structure Rule Description' )",
        "( 1.3.6.1.4.1.1466.115.121.1.18 DESC 'DL Submit Permission' )",
        "( 1.3.6.1.4.1.1466.115.121.1.19 DESC 'DSA Quality' )",
        "( 1.3.6.1.4.1.1466.115.121.1.20 DESC 'DSE Type' )",
        "( 1.3.6.1.4.1.1466.115.121.1.21 DESC 'Enhanced Guide' )",
        "( 1.3.6.1.4.1.1466.115.121.1.22 DESC 'Facsimile Telephone Number' )",
        "( 1.3.6.1.4.1.1466.115.121.1.23 DESC 'Fax' )",
        "( 1.3.6.1.4.1.1466.115.121.1.24 DESC 'Generalized Time' )",
        "( 1.3.6.1.4.1.1466.115.121.1.25 DESC 'Guide' )",
        "( 1.3.6.1.4.1.1466.115.121.1.26 DESC 'IA5 String' )",
        "( 1.3.6.1.4.1.1466.115.121.1.27 DESC 'Integer' )",
        "( 1.3.6.1.4.1.1466.115.121.1.28 DESC 'JPEG' )",
        "( 1.3.6.1.4.1.1466.115.121.1.29 DESC 'Master And Shadow Access Points' )",
        "( 1.3.6.1.4.1.1466.115.121.1.30 DESC 'Matching Rule Description' )",
        "( 1.3.6.1.4.1.1466.115.121.1.31 DESC 'Matching Rule Use Description' )",
        "( 1.3.6.1.4.1.1466.115.121.1.32 DESC 'Mail Preference' )",
        "( 1.3.6.1.4.1.1466.115.121.1.33 DESC 'Name And Optional UID' )",
        "( 1.3.6.1.4.1.1466.115.121.1.34 DESC 'Name Form Description' )",
        "( 1.3.6.1.4.1.1466.115.121.1.35 DESC 'Numeric String' )",
        "( 1.3.6.1.4.1.1466.115.121.1.36 DESC 'Object Class Description' )",
        "( 1.3.6.1.4.1.1466.115.121.1.37 DESC 'OID' )",
        "( 1.3.6.1.4.1.1466.115.121.1.38 DESC 'Other Mailbox' )",
        "( 1.3.6.1.4.1.1466.115.121.1.39 DESC 'Postal Address' )",
        "( 1.3.6.1.4.1.1466.115.121.1.40 DESC 'Protocol Information' )",
        "( 1.3.6.1.4.1.1466.115.121.1.41 DESC 'Presentation Address' )",
        "( 1.3.6.1.4.1.1466.115.121.1.42 DESC 'Printable String' )",
        "( 1.3.6.1.4.1.1466.115.121.1.43 DESC 'Substring Assertion' )",
        "( 1.3.6.1.4.1.1466.115.121.1.44 DESC 'Subtree Specification' )",
        "( 1.3.6.1.4.1.1466.115.121.1.45 DESC 'Supplier Information' )",
        "( 1.3.6.1.4.1.1466.115.121.1.46 DESC 'Supplier Or Consumer' )",
        "( 1.3.6.1.4.1.1466.115.121.1.47 DESC 'Supplier And Consumer' )",
        "( 1.3.6.1.4.1.1466.115.121.1.48 DESC 'Supported Algorithm' )",
        "( 1.3.6.1.4.1.1466.115.121.1.49 DESC 'Telephone Number' )",
        "( 1.3.6.1.4.1.1466.115.121.1.50 DESC 'Teletex Terminal Identifier' )",
        "( 1.3.6.1.4.1.1466.115.121.1.51 DESC 'Telex Number' )",
        "( 1.3.6.1.4.1.1466.115.121.1.52 DESC 'UTC Time' )",
    ];
    all_attributes.push(LdapPartialAttribute {
        atype: "ldapSyntaxes".to_string(),
        vals: ldap_syntaxes.into_iter().map(|s| s.into()).collect(),
    });

    // matchingRules: 匹配规则定义
    let matching_rules = vec![
        "( 2.5.13.2 NAME 'caseIgnoreMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.15 )",
        "( 2.5.13.4 NAME 'caseIgnoreOrderingMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.15 )",
        "( 2.5.13.5 NAME 'caseIgnoreSubstringsMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.58 )",
        "( 2.5.13.11 NAME 'integerMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.27 )",
        "( 2.5.13.12 NAME 'integerOrderingMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.27 )",
        "( 2.5.13.13 NAME 'numericStringMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.36 )",
        "( 2.5.13.14 NAME 'numericStringOrderingMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.36 )",
        "( 2.5.13.15 NAME 'numericStringSubstringsMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.58 )",
        "( 2.5.13.16 NAME 'caseExactMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.15 )",
        "( 2.5.13.17 NAME 'caseExactOrderingMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.15 )",
        "( 2.5.13.18 NAME 'caseExactSubstringsMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.58 )",
        "( 2.5.13.20 NAME 'telephoneNumberMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.50 )",
        "( 2.5.13.21 NAME 'telephoneNumberSubstringsMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.58 )",
        "( 2.5.13.22 NAME 'presentationAddressMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.43 )",
        "( 2.5.13.23 NAME 'uniqueMemberMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.34 )",
        "( 2.5.13.24 NAME 'protocolInformationMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.42 )",
        "( 2.5.13.27 NAME 'generalizedTimeMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.24 )",
        "( 2.5.13.28 NAME 'generalizedTimeOrderingMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.24 )",
        "( 2.5.13.29 NAME 'integerFirstComponentMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.27 )",
        "( 2.5.13.30 NAME 'objectIdentifierFirstComponentMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.38 )",
        "( 2.5.13.31 NAME 'directoryStringFirstComponentMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.15 )",
        "( 2.5.13.32 NAME 'wordMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.15 )",
        "( 2.5.13.33 NAME 'keywordMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.15 )",
        "( 1.3.6.1.4.1.1466.109.114.1 NAME 'caseIgnoreIA5Match' SYNTAX 1.3.6.1.4.1.1466.115.121.1.26 )",
        "( 1.3.6.1.4.1.1466.109.114.2 NAME 'caseIgnoreIA5SubstringsMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.58 )",
    ];
    all_attributes.push(LdapPartialAttribute {
        atype: "matchingRules".to_string(),
        vals: matching_rules.into_iter().map(|s| s.into()).collect(),
    });

    // matchingRuleUse: 匹配规则使用定义
    let matching_rule_use = vec![
        "( 2.5.13.2 APPLIES ( cn $ description $ name $ o $ ou $ sn $ st $ l $ street $ title $ givenName $ initials $ generationQualifier $ dnQualifier $ displayName $ businessCategory $ postalCode $ postalAddress $ telephoneNumber $ facsimileTelephoneNumber $ seeAlso $ member $ owner $ roleOccupant $ secretary $ x121Address $ registeredAddress $ destinationIndicator $ preferredDeliveryMethod $ telexNumber $ teletexTerminalIdentifier $ internationaliSDNNumber $ physicalDeliveryOfficeName $ streetAddress $ postOfficeBox $ st $ l $ postalCode $ postalAddress $ userPassword $ searchGuide $ teletexTerminalIdentifier $ facsimileTelephoneNumber $ x121Address $ internationaliSDNNumber $ registeredAddress $ destinationIndicator $ preferredDeliveryMethod $ telexNumber $ physicalDeliveryOfficeName $ streetAddress $ postOfficeBox $ st $ l $ postalCode $ postalAddress $ userPassword $ searchGuide $ seeAlso $ businessCategory $ x121Address $ registeredAddress $ destinationIndicator $ preferredDeliveryMethod $ telexNumber $ teletexTerminalIdentifier $ internationaliSDNNumber $ facsimileTelephoneNumber $ physicalDeliveryOfficeName $ streetAddress $ postOfficeBox $ st $ l $ postalCode $ postalAddress $ userPassword $ searchGuide $ seeAlso $ businessCategory $ x121Address $ registeredAddress $ destinationIndicator $ preferredDeliveryMethod $ telexNumber $ teletexTerminalIdentifier $ internationaliSDNNumber $ facsimileTelephoneNumber $ physicalDeliveryOfficeName $ streetAddress $ postOfficeBox $ st $ l $ postalCode $ postalAddress $ userPassword $ searchGuide $ seeAlso $ businessCategory ) )",
        "( 2.5.13.4 APPLIES ( cn $ description $ name $ o $ ou $ sn $ st $ l $ street $ title $ givenName $ initials $ generationQualifier $ dnQualifier $ displayName $ businessCategory $ postalCode $ postalAddress $ telephoneNumber $ facsimileTelephoneNumber $ seeAlso $ member $ owner $ roleOccupant $ secretary ) )",
        "( 2.5.13.5 APPLIES ( cn $ description $ name $ o $ ou $ sn $ st $ l $ street $ title $ givenName $ initials $ generationQualifier $ dnQualifier $ displayName $ businessCategory $ postalCode $ postalAddress $ telephoneNumber $ facsimileTelephoneNumber $ seeAlso $ member $ owner $ roleOccupant $ secretary ) )",
        "( 2.5.13.11 APPLIES ( uidNumber $ gidNumber ) )",
        "( 2.5.13.12 APPLIES ( uidNumber $ gidNumber ) )",
        "( 2.5.13.13 APPLIES ( telephoneNumber $ facsimileTelephoneNumber ) )",
        "( 2.5.13.14 APPLIES ( telephoneNumber $ facsimileTelephoneNumber ) )",
        "( 2.5.13.15 APPLIES ( telephoneNumber $ facsimileTelephoneNumber ) )",
        "( 2.5.13.16 APPLIES ( serialNumber ) )",
        "( 2.5.13.17 APPLIES ( serialNumber ) )",
        "( 2.5.13.18 APPLIES ( serialNumber ) )",
        "( 2.5.13.20 APPLIES ( telephoneNumber $ facsimileTelephoneNumber ) )",
        "( 2.5.13.21 APPLIES ( telephoneNumber $ facsimileTelephoneNumber ) )",
        "( 2.5.13.27 APPLIES ( createTimestamp $ modifyTimestamp ) )",
        "( 2.5.13.28 APPLIES ( createTimestamp $ modifyTimestamp ) )",
        "( 1.3.6.1.4.1.1466.109.114.1 APPLIES ( mail $ rfc822Mailbox ) )",
        "( 1.3.6.1.4.1.1466.109.114.2 APPLIES ( mail $ rfc822Mailbox ) )",
    ];
    all_attributes.push(LdapPartialAttribute {
        atype: "matchingRuleUse".to_string(),
        vals: matching_rule_use.into_iter().map(|s| s.into()).collect(),
    });

    // createTimestamp 和 modifyTimestamp: 时间戳（使用当前时间）
    // LDAP 时间戳格式：YYYYMMDDHHmmssZ (GeneralizedTime)
    // 使用固定时间戳，因为 schema 通常不会频繁变化
    let timestamp = "20250101000000Z".to_string();
    all_attributes.push(LdapPartialAttribute {
        atype: "createTimestamp".to_string(),
        vals: vec![timestamp.clone().into()],
    });
    all_attributes.push(LdapPartialAttribute {
        atype: "modifyTimestamp".to_string(),
        vals: vec![timestamp.into()],
    });

    // 根据请求的属性列表过滤属性
    filter_attributes_by_request(&all_attributes, &query.attributes)
}

// 判断search时是否返回域节点
pub fn should_return_domain_level_in_search(_level: LdapBaseDnLevel, _scope: LdapSearchScope) -> bool {
    false
}

// 判断search时是否返回OU节点
pub fn should_return_ou_level_in_search(level: LdapBaseDnLevel, scope: LdapSearchScope) -> bool {
    matches!(level, LdapBaseDnLevel::Domain)
        && (matches!(scope, LdapSearchScope::OneLevel) || matches!(scope, LdapSearchScope::Subtree) || matches!(scope, LdapSearchScope::Children))
}
