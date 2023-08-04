use super::{SgFilterIpTime, SgFilterIpTimeConfig};

use tardis::chrono::NaiveTime;

use tardis::serde_json;
#[test]
fn parse_config() {
    let json_str = include_str!("./testconfig.json");
    let value: Vec<SgFilterIpTimeConfig> = serde_json::from_str(json_str).expect("fail to parse config into json");
    let filters = value.into_iter().map(SgFilterIpTime::from).collect::<Vec<_>>();
    let rule_0_0 = &filters[0].rules[0].1;
    assert!(!rule_0_0.check_by_time(NaiveTime::from_hms_opt(23, 59, 59).expect("invalid native time")));
    dbg!(filters);
}
