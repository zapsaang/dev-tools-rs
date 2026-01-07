use clap::Args;
use crate::utils::{read_input, is_alfred_env, print_alfred, AlfredItem};
use chrono::{DateTime, Utc, Local, TimeZone, NaiveDateTime};

#[derive(Args)]
pub struct TsArgs {
    /// 输入时间：可以是时间戳、日期字符串、或者 'now'
    #[arg(value_name = "INPUT")]
    input: Option<Vec<String>>,

    /// 强制 Alfred 模式
    #[arg(long)]
    alfred: bool,
}

// 尝试解析各种格式的时间字符串
fn parse_datetime(input: &str) -> Option<DateTime<Utc>> {
    let input = input.trim();

    // 1. 处理 "now" 或空
    if input.eq_ignore_ascii_case("now") || input.is_empty() {
        return Some(Utc::now());
    }

    // 2. 尝试解析为数字 (时间戳)
    if let Ok(ts) = input.parse::<i64>() {
        // 猜测是秒(10位左右) 还是 毫秒(13位左右)
        // 3000000000 秒大约是 2065 年，以此为界限区分
        if ts > 3_000_000_000 {
            return Utc.timestamp_millis_opt(ts).single();
        } else {
            return Utc.timestamp_opt(ts, 0).single();
        }
    }

    // 3. 尝试标准 ISO 8601 / RFC 3339 (e.g. 2023-01-01T12:00:00Z)
    if let Ok(dt) = DateTime::parse_from_rfc3339(input) {
        return Some(dt.with_timezone(&Utc));
    }

    // 4. 尝试常见格式 "YYYY-MM-DD HH:mm:ss" (默认为本地时间)
    let common_formats = vec![
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d",
    ];

    for fmt in common_formats {
        // 尝试解析为 Naive (无时区)，然后加上本地时区
        if let Ok(naive) = NaiveDateTime::parse_from_str(input, fmt) {
            // 这里假设输入的是本地时间
            return Local.from_local_datetime(&naive).single().map(|d| d.with_timezone(&Utc));
        }
        
        // 特殊处理只有日期的情况，补全时间
        if fmt == "%Y-%m-%d" {
             if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(input, fmt) {
                 return Local.from_local_datetime(&naive_date.and_hms_opt(0, 0, 0).unwrap())
                    .single()
                    .map(|d| d.with_timezone(&Utc));
             }
        }
    }

    // 5. 仅时间："HH:mm:ss"，默认当天本地日期
    if let Ok(t) = chrono::NaiveTime::parse_from_str(input, "%H:%M:%S") {
        let today = Local::now().date_naive();
        let naive = today.and_time(t);
        return Local.from_local_datetime(&naive).single().map(|d| d.with_timezone(&Utc));
    }

    None
}

// 计算相对时间 (简单实现)
fn relative_time(dt: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(*dt);
    
    let seconds = diff.num_seconds();
    let abs_seconds = seconds.abs();

    let time_str = if abs_seconds < 60 {
        format!("{}s", abs_seconds)
    } else if abs_seconds < 3600 {
        format!("{}m", abs_seconds / 60)
    } else if abs_seconds < 86400 {
        format!("{}h", abs_seconds / 3600)
    } else {
        format!("{}d", abs_seconds / 86400)
    };

    if seconds > 0 {
        format!("{} ago", time_str)
    } else if seconds < 0 {
        format!("in {}", time_str)
    } else {
        "just now".to_string()
    }
}

pub fn run(args: TsArgs) {
    let raw_input = read_input(args.input);
    let input_str = if raw_input.is_empty() { "now" } else { &raw_input };

    match parse_datetime(input_str) {
        Some(utc_dt) => {
            let local_dt = utc_dt.with_timezone(&Local);
            let ts_secs = utc_dt.timestamp();
            let ts_millis = utc_dt.timestamp_millis();
            let rel_time = relative_time(&utc_dt);

            // 准备输出数据
            let mut items = Vec::new();

            // 1. 本地格式 (最常用)
            let local_str = local_dt.format("%Y-%m-%d %H:%M:%S").to_string();
            items.push(AlfredItem::new("local", &local_str, "Local Time", &local_str));

            // 2. 时间戳 (秒)
            items.push(AlfredItem::new(
                "sec",
                &ts_secs.to_string(),
                "Timestamp (Seconds)",
                &ts_secs.to_string(),
            ));

            // 3. 时间戳 (毫秒)
            items.push(AlfredItem::new(
                "ms",
                &ts_millis.to_string(),
                "Timestamp (Millis)",
                &ts_millis.to_string(),
            ));

            // 4. UTC ISO 格式
            let utc_str = utc_dt.to_rfc3339(); // e.g. 2023-01-01T00:00:00+00:00
            items.push(AlfredItem::new("utc", &utc_str, "UTC (ISO 8601)", &utc_str));

            // 5. 相对时间
            items.push(AlfredItem::new("rel", &rel_time, "Relative Time", &rel_time));

            // 输出
            if is_alfred_env(args.alfred) {
                print_alfred(items);
            } else {
                println!("Input: {}", input_str);
                println!("Parsed: {}", local_dt);
                println!("---");
                for item in items {
                    println!("{:<20} {}", item.subtitle.to_string() + ":", item.arg);
                }
            }
        },
        None => {
            let err_msg = format!("Invalid date format: {}", input_str);
            if is_alfred_env(args.alfred) {
                let items = vec![AlfredItem::new("error", "Error", &err_msg, &err_msg)];
                print_alfred(items);
            } else {
                eprintln!("{}", err_msg);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_now_and_timestamps() {
        assert!(parse_datetime("now").is_some());

        let secs = parse_datetime("1700000000").expect("secs");
        assert_eq!(secs.timestamp(), 1_700_000_000);

        let ms = parse_datetime("1700000000000").expect("ms");
        assert_eq!(ms.timestamp_millis(), 1_700_000_000_000);
    }

    #[test]
    fn parse_rfc3339() {
        let dt = parse_datetime("2023-01-01T00:00:00Z").expect("rfc3339");
        assert_eq!(dt.timestamp(), 1_672_531_200);
    }

    #[test]
    fn parse_time_only() {
        // 只验证能解析成功即可（日期依赖本地时区与当天）
        assert!(parse_datetime("12:34:56").is_some());
    }

    #[test]
    fn relative_time_signs() {
        let now = Utc::now();
        let past = now - chrono::Duration::seconds(10);
        let future = now + chrono::Duration::seconds(10);

        let past_s = relative_time(&past);
        let future_s = relative_time(&future);

        assert!(past_s.contains("ago"));
        assert!(future_s.contains("in"));
    }
}