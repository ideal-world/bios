use std::net::IpAddr;

use super::{SgFilterIpTime, SgFilterIpTimeConfig};

use tardis::{chrono::Local, serde_json};
#[test]
fn parse_config() {
    let json_str = include_str!("./testconfig.json");
    println!("Init ip-time plugin, local timezone offset: {tz}", tz = Local::now().offset());
    let value: Vec<SgFilterIpTimeConfig> = serde_json::from_str(json_str).expect("fail to parse config into json");
    let filters = value.into_iter().map(SgFilterIpTime::from).collect::<Vec<_>>();
    let ip: IpAddr = "123.123.123.123".parse().expect("invalid ip");
    let passed = filters[0].check_ip(&ip);
    assert!(!passed);
    dbg!(filters);
}
