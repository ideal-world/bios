// Placeholder $bios{KEY}

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

use bios_sdk_invoke::clients::iam_client::{IamCertDecodeRequest, IamClient};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::regex::Regex;
use tardis::tardis_static;

use crate::conf_config::ConfConfig;

use super::ConfigDescriptor;

tardis_static! {
    pub place_holder_regex: Regex = Regex::new(r"\$(CERT|ENV)\{(.*?)\}").expect("invalid content replace regex");
}

#[derive(Debug, Clone, Copy)]
enum Segment<'s> {
    Raw(&'s str),
    CertReplace { key: &'s str },
    EnvReplace { key: &'s str },
}

fn parse_content(content: &str) -> Vec<Segment<'_>> {
    let captures = place_holder_regex().captures_iter(content);
    let mut idx = 0;
    let mut result = Vec::new();
    for capture in captures {
        let replace = capture.get(0).expect("capture error");
        let kind = capture.get(1).expect("capture error");
        let key = capture.get(2).expect("capture error");
        result.push(Segment::Raw(&content[idx..replace.start()]));
        match kind.as_str() {
            "CERT" => result.push(Segment::CertReplace { key: key.as_str() }),
            "ENV" => result.push(Segment::EnvReplace { key: key.as_str() }),
            _ => result.push(Segment::Raw(replace.as_str())),
        }
        idx = replace.end();
    }
    result.push(Segment::Raw(&content[idx..]));
    result
}

#[derive(Debug)]
pub struct RenderPolicy {
    pub render_cert: bool,
    pub render_env: bool,
}
pub async fn render_content(
    descriptor: &ConfigDescriptor,
    content: String,
    config: &ConfConfig,
    policy: RenderPolicy,
    funs: &tardis::TardisFunsInst,
    ctx: &TardisContext,
) -> TardisResult<String> {
    let segments = parse_content(&content);
    // render
    let cert_keys = segments.iter().fold(HashSet::new(), |mut set, seg| {
        if let Segment::CertReplace { key } = seg {
            set.insert(*key);
        }
        set
    });
    let env_keys = segments.iter().fold(HashSet::new(), |mut set, seg| {
        if let Segment::EnvReplace { key } = seg {
            set.insert(*key);
        }
        set
    });
    let kv_cert_map = if cert_keys.is_empty() || !policy.render_cert {
        HashMap::new()
    } else {
        get_cert_kvmap(cert_keys, config, funs, ctx).await.unwrap_or_default()
    };

    let kv_env_map = if env_keys.is_empty() || !policy.render_env {
        HashMap::new()
    } else {
        get_env_kvmap(&descriptor.namespace_id, env_keys, config, funs, ctx).await.unwrap_or_default()
    };

    let content = segments.into_iter().fold(String::new(), |content, seg| match seg {
        Segment::Raw(raw) => content + raw,
        Segment::CertReplace { key } => content + kv_cert_map.get(key).map(String::as_str).unwrap_or(key),
        Segment::EnvReplace { key } => content + kv_env_map.get(key).map(String::as_str).unwrap_or_default(),
    });
    Ok(content)
}

async fn get_cert_kvmap(codes: HashSet<&str>, config: &ConfConfig, funs: &tardis::TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, String>> {
    let url = config.iam_client.base_url.as_str();
    let client = IamClient::new("", funs, ctx, url);
    let codes = codes.into_iter().map(|s| s.to_string()).collect::<HashSet<String>>();
    let req = IamCertDecodeRequest { codes };
    let response = client.batch_decode_cert(&req).await?;
    Ok(response)
}
async fn get_env_kvmap(namespace: &str, codes: HashSet<&str>, config: &ConfConfig, funs: &tardis::TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, String>> {
    let mut env_config_descriptor = ConfigDescriptor {
        namespace_id: namespace.to_string(),
        group: String::new(),
        data_id: config.namespace_env_config.to_string(),
        ..Default::default()
    };
    thread_local! {
        static MD5: RefCell<String> = "".to_string().into();
        static MAP: RefCell<HashMap<String, String>> = HashMap::new().into();
    };
    let db_md5: String = super::get_md5(&mut env_config_descriptor, funs, ctx).await?;
    let md5_equal = MD5.with(|md5| md5.borrow().as_str() == db_md5.as_str());
    let extract = || {
        MAP.with_borrow(|map| {
            codes.iter().fold(HashMap::default(), |mut collect_map, key| {
                if let Some(val) = map.get(*key) {
                    collect_map.insert(key.to_string(), val.to_string());
                }
                collect_map
            })
        })
    };
    if md5_equal {
        Ok(extract())
    } else {
        let config: String = super::get_config(&mut env_config_descriptor, funs, ctx).await?;
        let map_now = crate::utils::dot_env_parser(&config);
        MAP.with(|map| map.replace(map_now.clone()));
        MD5.with(|md5| md5.replace(db_md5));
        Ok(extract())
    }
}
pub fn has_placeholder_auth(source_addr: IpAddr, funs: &tardis::TardisFunsInst) -> bool {
    let cfg = funs.conf::<ConfConfig>();
    cfg.placeholder_white_list.iter().any(|net| net.contains(&source_addr))
}

pub async fn render_content_for_ip(
    raw_descriptor: &ConfigDescriptor,
    content: String,
    source_addr: IpAddr,
    funs: &tardis::TardisFunsInst,
    ctx: &tardis::basic::dto::TardisContext,
) -> TardisResult<String> {
    let cfg = funs.conf::<ConfConfig>();
    // let level = get_scope_level_by_context(ctx)?;
    let render_cert = has_placeholder_auth(source_addr, funs);
    let render_env = true;
    let render_policy = RenderPolicy { render_cert, render_env };
    tardis::tracing::trace!(?source_addr, ?ctx, ?render_policy, "[BIOS.Spi-Config] Trying to render config");
    render_content(raw_descriptor, content, cfg.as_ref(), RenderPolicy { render_cert, render_env }, funs, ctx).await
}

#[test]
#[cfg(test)]
fn test() {
    let test_config = r#"
The Code is $CERT{CODE} and the value is $CERT{VALUE}
The Code is $ENV{CODE} and the value is $ENV{VALUE}
L
"#;
    let segs = parse_content(test_config);
    println!("{:?}", segs);
    assert_eq!(segs.len(), 9);
}
