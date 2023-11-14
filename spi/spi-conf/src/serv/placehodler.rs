// PlaceHodler $bios{KEY}

use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

use bios_sdk_invoke::clients::iam_client::{IamClient, IamCertDecodeRequest};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::regex::Regex;
use tardis::tardis_static;

use crate::conf_config::ConfConfig;
use crate::dto::conf_config_dto::ConfigDescriptor;

use super::get_config;
tardis_static! {
    pub place_holder_regex: Regex = Regex::new(r"\$bios\{([A-Z_]+)\}").expect("invalid content replace regex");
}

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
    // no need for render
    if segments.len() == 1 {
        return Ok(content);
    }
    // render
    let keys = segments.iter().fold(HashSet::new(), |mut set, seg| {
        if let Segment::Replace { key } = seg {
            set.insert(*key);
        }
        set
    });
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
    let key = config.iam_client.cert_encode_key.clone();
    let req = IamCertDecodeRequest {
        key,
        codes,
    };
    let response = client.batch_decode_cert(&req).await?;
    Ok(response)
}

pub fn has_placeholder_auth(source_addr: IpAddr, funs: &tardis::TardisFunsInst) -> bool {
    let cfg = funs.conf::<ConfConfig>();
    cfg.placeholder_white_list.contains(&source_addr)
}

pub async fn get_rendered_config(
    descriptor: &mut ConfigDescriptor,
    source_addr: IpAddr,
    funs: &tardis::TardisFunsInst,
    ctx: &tardis::basic::dto::TardisContext,
) -> TardisResult<String> {
    let config = get_config(descriptor, funs, ctx).await?;
    let cfg = funs.conf::<ConfConfig>();
    if has_placeholder_auth(source_addr, funs) {
        rander_content(config, cfg.as_ref(), funs, ctx).await
    } else {
        Ok(config)
    }
}

// #[test]
// #[cfg(test)]
// fn test() {
//     use tardis::crypto::crypto_key::TardisCryptoKey;
//     let key = TardisCryptoKey.rand_32_bytes();
//     let id = tardis::TardisFuns::field.nanoid();
//     let owner = "BIOS";
//     let ph = encode_placeholder(&id, owner, &key).expect("encode failed");
//     println!("ph: {}", ph);
//     let (_id, _owner) = decode_placeholder(&ph, &key).expect("fail to decode");
//     println!("id: {}, owner: {}", id, owner);
//     assert_eq!(id, _id);
//     assert_eq!(owner, _owner);
// }
