use std::time::Duration;

use spacegate_shell::BoxError;

pub fn parse_duration(duration: &str) -> Result<Duration, BoxError> {
    // (<number><unit>)*
    // <number> = [0-9]+
    // <unit> = "ns" | "us" | "ms" | "s" | "m" | "h" | "d"
    let mut duration = duration;
    let mut total_duration = Duration::new(0, 0);
    while !duration.is_empty() {
        // Parse number
        let (number, rest) = match duration.find(|c: char| !c.is_ascii_digit() && c != ' ') {
            Some(index) => duration.split_at(index),
            None => (duration, ""),
        };
        let number = number.trim().parse::<u64>()?;
        duration = rest;

        // Parse unit
        let (unit, rest) = match duration.find(|c: char| !c.is_alphabetic() && c != ' ') {
            Some(index) => duration.split_at(index),
            None => (duration, ""),
        };

        let unit = match unit.trim() {
            "ns" | "nanosecond" => Duration::from_nanos(number),
            "us" | "microsecond" => Duration::from_micros(number),
            "ms" | "millisecond" => Duration::from_millis(number),
            "s" | "second" => Duration::from_secs(number),
            "m" | "min" => Duration::from_secs(number * 60),
            "h" | "hour" => Duration::from_secs(number * 60 * 60),
            "d" | "day" => Duration::from_secs(number * 60 * 60 * 24),
            _ => return Err(format!("Invalid unit: {}", unit).into()),
        };

        total_duration += unit;
        duration = rest;
    }
    Ok(total_duration)
}

#[cfg(test)]
mod test {
    #[test]
    fn test_duration_parse() {
        use std::time::Duration;
        macro_rules! test {
            ($(
                $str: literal => $duration: expr,
            )*) => {
                $(
                    let duration = super::parse_duration($str).unwrap();
                    assert_eq!(duration, $duration);
                )*
            };
        }
        test! {
            "1ns" => Duration::from_nanos(1),
            "1us" => Duration::from_micros(1),
            "1ms" => Duration::from_millis(1),
            "1s" => Duration::from_secs(1),
            "1m" => Duration::from_secs(60),
            "1h" => Duration::from_secs(60 * 60),
            "1d" => Duration::from_secs(60 * 60 * 24),
            "1ns1us1ms1s1m1h1d" => Duration::from_nanos(1) + Duration::from_micros(1) + Duration::from_millis(1) + Duration::from_secs(1) + Duration::from_secs(60) + Duration::from_secs(60 * 60) + Duration::from_secs(60 * 60 * 24),
            "1d 1h 1min 1s 1ms 1us 1ns" => Duration::from_secs(60 * 60 * 24) + Duration::from_secs(60 * 60) + Duration::from_secs(60) + Duration::from_secs(1) + Duration::from_millis(1) + Duration::from_micros(1) + Duration::from_nanos(1),
        }
    }
}
