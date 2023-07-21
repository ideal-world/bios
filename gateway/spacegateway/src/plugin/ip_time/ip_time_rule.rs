use std::{default, ops::Range, time::SystemTime};

use serde::{Deserialize, Serialize};
use tardis::chrono::{Local, NaiveTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum IpTimeRule {
    BlackList { ban: Vec<Range<NaiveTime>> },
    WhiteList { allow: Vec<Range<NaiveTime>> },
}

impl IpTimeRule {
    pub fn check_by_time(&self, time: NaiveTime) -> bool {
        let contains_time = |range: &Range<NaiveTime>| {
            if range.start > range.end {
                !(range.end..range.start).contains(&time)
            } else {
                range.contains(&time)
            }
        };
        match self {
            IpTimeRule::WhiteList { allow } => allow.iter().any(contains_time),
            IpTimeRule::BlackList { ban } => !ban.iter().any(contains_time),
        }
    }
    pub fn check_by_now(&self) -> bool {
        self.check_by_time(Local::now().time())
    }
}
