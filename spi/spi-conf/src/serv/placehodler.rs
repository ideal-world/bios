// PlaceHodler $bios{KEY}

use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

use bios_basic::rbum::helper::rbum_scope_helper::get_scope_level_by_context;
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_sdk_invoke::clients::iam_client::{IamCertDecodeRequest, IamClient};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::regex::Regex;
use tardis::tardis_static;

use crate::conf_config::ConfConfig;

tardis_static! {
    pub place_holder_regex: Regex = Regex::new(r"\$CERT\{([A-Z_]+)\}").expect("invalid content replace regex");
}

#[derive(Debug, Clone, Copy)]
enum Segment<'s> {
    Raw(&'s str),
    Replace { key: &'s str },
}

fn parse_content(content: &str) -> Vec<Segment<'_>> {
    let mut new_content = String::new();
    let matcher = place_holder_regex().find_iter(content);
    let mut idx = 0;
    let mut result = Vec::new();
    for mat in matcher {
        result.push(Segment::Raw(&content[idx..mat.start()]));
        let key = &content[(mat.start() + 6)..(mat.end() - 1)];
        result.push(Segment::Replace { key });
        idx = mat.end();
    }
    new_content.push_str(&content[idx..]);
    result
}

pub async fn rander_content(content: String, config: &ConfConfig, funs: &tardis::TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
    let segments = parse_content(&content);
    // render
    let keys = segments.iter().fold(HashSet::new(), |mut set, seg| {
        if let Segment::Replace { key } = seg {
            set.insert(*key);
        }
        set
    });
    // no need for render
    if keys.is_empty() {
        return Ok(content);
    }
    // enhancement: this can be depart from function, KvSource should be trait
    let kvmap = get_kvmap(keys, config, funs, ctx).await?;
    let content = segments.into_iter().fold(String::new(), |content, seg| match seg {
        Segment::Raw(raw) => content + raw,
        Segment::Replace { key } => content + kvmap.get(key).unwrap_or(&String::new()).as_str(),
    });
    Ok(content)
}

async fn get_kvmap(codes: HashSet<&str>, config: &ConfConfig, funs: &tardis::TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, String>> {
    let url = config.iam_client.base_url.as_str();
    let client = IamClient::new("", funs, ctx, url);
    let codes = codes.into_iter().map(|s| s.to_string()).collect::<HashSet<String>>();
    let req = IamCertDecodeRequest { codes };
    let response = client.batch_decode_cert(&req).await?;
    Ok(response)
}

pub fn has_placeholder_auth(source_addr: IpAddr, funs: &tardis::TardisFunsInst) -> bool {
    let cfg = funs.conf::<ConfConfig>();
    cfg.placeholder_white_list.contains(&source_addr)
}

pub async fn render_content_for_ip(content: String, source_addr: IpAddr, funs: &tardis::TardisFunsInst, ctx: &tardis::basic::dto::TardisContext) -> TardisResult<String> {
    let cfg = funs.conf::<ConfConfig>();
    let level = get_scope_level_by_context(ctx)?;
    if has_placeholder_auth(source_addr, funs) && level == RbumScopeLevelKind::Root {
        rander_content(content, cfg.as_ref(), funs, ctx).await
    } else {
        Ok(content)
    }
}

#[test]
#[cfg(test)]
fn test() {
    let test_config = r#"
The Code is $CERT{CODE} and the value is $CERT{VALUE}
"#;
    parse_content(test_config);
}
