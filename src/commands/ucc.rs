use crate::utils::{AlfredItem, is_alfred_env, print_alfred, read_input};
use base64::Engine as _;
use base64::engine::general_purpose;
use clap::Args;
use std::borrow::Cow;

#[derive(Args)]
pub struct UccArgs {
    #[arg(value_name = "INPUT")]
    input: Option<Vec<String>>,

    #[arg(short, long)]
    format: Option<String>,

    #[arg(long)]
    alfred: bool,

    #[arg(short, long)]
    json: bool,

    #[arg(short, long)]
    quiet: bool,
}

#[derive(Debug, PartialEq)]
enum InputType {
    Jwt,        // 新增: eyJhbG...
    HtmlEntity, // 新增: &lt;
    UnicodePoints,
    UnicodeEscaped,
    Hex,
    Url,
    Base64,
    Text,
}

fn detect_type(input: &str) -> InputType {
    let trimmed = input.trim();

    // 1. JWT (header.payload.signature)
    if trimmed.split('.').count() == 3 && trimmed.len() > 20 {
        // 简单验证 payload 部分是否为 base64
        let parts: Vec<&str> = trimmed.split('.').collect();
        if decode_base64_forgiving(parts[1]).is_ok() {
            return InputType::Jwt;
        }
    }

    // 3. HTML Entity
    if trimmed.starts_with('&') && trimmed.ends_with(';') {
        return InputType::HtmlEntity;
    }

    // 4. Unicode Points
    if trimmed.to_uppercase().starts_with("U+") {
        let plain = trimmed.replace("U+", "").replace("u+", "").replace(" ", "");
        if plain.chars().all(|c| c.is_ascii_hexdigit()) {
            return InputType::UnicodePoints;
        }
    }

    // 5. Unicode Escaped
    if trimmed.contains("\\u") {
        return InputType::UnicodeEscaped;
    }

    // 6. URL Encoded
    if trimmed.contains('%') {
        if let Ok(decoded) = urlencoding::decode(trimmed) {
            if decoded != trimmed {
                return InputType::Url;
            }
        }
    }

    // 7. Hex (Hex 解码后必须是有效文本，否则视为 Text)
    if trimmed.len() > 2 && trimmed.len() % 2 == 0 && trimmed.chars().all(|c| c.is_ascii_hexdigit())
    {
        if let Ok(bytes) = hex::decode(trimmed) {
            if std::str::from_utf8(&bytes).is_ok() {
                return InputType::Hex;
            }
        }
    }

    // 8. Base64
    if trimmed.len() > 4 && trimmed.len() % 4 == 0 {
        let is_b64 = trimmed.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=' || c == '-' || c == '_'
        }); // URL safe base64
        if is_b64 {
            if let Ok(bytes) = decode_base64_forgiving(trimmed) {
                if std::str::from_utf8(&bytes).is_ok() {
                    // 过滤掉短的普通单词误判
                    if trimmed.contains('=') || trimmed.len() > 8 {
                        return InputType::Base64;
                    }
                }
            }
        }
    }

    InputType::Text
}

fn decode_base64_forgiving(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
    let trimmed = input.trim();
    // URL-safe 的 JWT payload 经常没有 padding，这里做一个宽松补齐
    let rem = trimmed.len() % 4;
    let padded = if rem == 0 {
        trimmed.to_string()
    } else {
        format!("{}{}", trimmed, "=".repeat(4 - rem))
    };

    general_purpose::URL_SAFE
        .decode(&padded)
        .or_else(|_| general_purpose::STANDARD.decode(&padded))
}

fn convert(input: &str, input_type: &InputType) -> (String, String) {
    match input_type {
        InputType::Jwt => {
            let parts: Vec<&str> = input.split('.').collect();
            // Decode payload (part 2)
            // 需要处理 URL Safe Base64 并且可能没有 padding
            let payload = parts[1];
            if let Ok(bytes) = decode_base64_forgiving(payload) {
                if let Ok(json) = String::from_utf8(bytes) {
                    // 格式化一下 JSON
                    let pretty = serde_json::from_str::<serde_json::Value>(&json)
                        .map(|v| serde_json::to_string_pretty(&v).unwrap())
                        .unwrap_or(json);
                    return (pretty, "JWT Payload".into());
                }
            }
            ("Invalid JWT Payload".into(), "Error".into())
        }
        InputType::HtmlEntity => (
            html_escape::decode_html_entities(input).to_string(),
            "HTML实体 → 字符".into(),
        ),
        InputType::UnicodePoints => {
            let mut res = String::new();
            let cleaned = input
                .replace("U+", " ")
                .replace("u+", " ")
                .replace('U', " ")
                .replace('u', " ")
                .replace('+', " ");

            for part in cleaned.split_whitespace() {
                if let Ok(code) = u32::from_str_radix(part, 16) {
                    if let Some(c) = std::char::from_u32(code) {
                        res.push(c);
                    }
                }
            }
            (res, "Unicode码点 → 字符".into())
        }
        InputType::UnicodeEscaped => {
            let json_str = format!("\"{}\"", input);
            let res =
                serde_json::from_str::<String>(&json_str).unwrap_or_else(|_| input.to_string());
            (res, "Unicode转义序列 → 字符串".into())
        }
        InputType::Hex => {
            if let Ok(bytes) = hex::decode(input.trim()) {
                (
                    String::from_utf8_lossy(&bytes).to_string(),
                    "UTF-8十六进制 → 字符串".into(),
                )
            } else {
                ("Error".into(), "Error".into())
            }
        }
        InputType::Url => (
            urlencoding::decode(input)
                .unwrap_or(Cow::Borrowed(input))
                .to_string(),
            "URL编码 → 字符串".into(),
        ),
        InputType::Base64 => {
            if let Ok(bytes) = decode_base64_forgiving(input) {
                (
                    String::from_utf8_lossy(&bytes).to_string(),
                    "Base64编码 → 字符串".into(),
                )
            } else {
                ("Error".into(), "Error".into())
            }
        }
        InputType::Text => (input.to_string(), "文本".into()),
    }
}

fn generate_all_encodings(input: &str) -> Vec<AlfredItem> {
    let mut items = Vec::new();

    // Text -> Encodings
    let unicode_pts: String = input
        .chars()
        .map(|c| format!("U+{:04X}", c as u32))
        .collect::<Vec<_>>()
        .join(" ");
    items.push(AlfredItem::new(
        "unicode",
        &unicode_pts,
        "Unicode码点",
        &unicode_pts,
    ));

    let hex_str = hex::encode(input).to_uppercase();
    items.push(AlfredItem::new("hex", &hex_str, "UTF-8十六进制", &hex_str));

    let url_str = urlencoding::encode(input).to_string();
    items.push(AlfredItem::new("url", &url_str, "URL编码", &url_str));

    let b64_str = general_purpose::STANDARD.encode(input);
    items.push(AlfredItem::new("base64", &b64_str, "Base64编码", &b64_str));

    let html_str = html_escape::encode_text(input).to_string();
    items.push(AlfredItem::new("html", &html_str, "HTML转义", &html_str));

    items
}

pub fn run(args: UccArgs) {
    let input_str = read_input(args.input);
    if input_str.is_empty() {
        return;
    }

    let input_type = detect_type(&input_str);
    let is_alfred =
        is_alfred_env(args.alfred) && !args.json && !args.quiet && args.format.is_none();

    if is_alfred {
        let items = if input_type == InputType::Text {
            generate_all_encodings(&input_str)
        } else {
            let (res, info) = convert(&input_str, &input_type);
            vec![AlfredItem::new("result", &res, &info, &res)]
        };
        print_alfred(items);
        return;
    }

    // CLI 输出模式
    if input_type == InputType::Text && args.format.is_none() {
        // 展示所有编码
        let items = generate_all_encodings(&input_str);
        if args.json {
            print_alfred(items); // 复用 Alfred JSON 结构作为 JSON 输出
        } else {
            println!("Input: {}", input_str);
            for item in items {
                println!("{}: {}", item.subtitle, item.arg);
            }
        }
    } else {
        // 解码模式
        let (res, info) = convert(&input_str, &input_type);
        if args.json {
            println!(
                "{}",
                serde_json::json!({"result": res, "info": info}).to_string()
            );
        } else if args.quiet {
            print!("{}", res);
        } else {
            println!("Detected: {:?}", input_type);
            println!("Conversion: {}", info);
            println!("Result:\n{}", res);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose;

    #[test]
    fn detect_and_convert_hex() {
        let input = "48656C6C6F";
        assert_eq!(detect_type(input), InputType::Hex);
        let (out, _) = convert(input, &InputType::Hex);
        assert_eq!(out, "Hello");
    }

    #[test]
    fn detect_and_convert_base64() {
        let input = "aGVsbG8=";
        assert_eq!(detect_type(input), InputType::Base64);
        let (out, _) = convert(input, &InputType::Base64);
        assert_eq!(out, "hello");
    }

    #[test]
    fn detect_and_convert_url() {
        let input = "%E4%BD%A0%E5%A5%BD";
        assert_eq!(detect_type(input), InputType::Url);
        let (out, _) = convert(input, &InputType::Url);
        assert_eq!(out, "你好");
    }

    #[test]
    fn detect_and_convert_html_entity() {
        let input = "&lt;div&gt;";
        assert_eq!(detect_type(input), InputType::HtmlEntity);
        let (out, _) = convert(input, &InputType::HtmlEntity);
        assert_eq!(out, "<div>");
    }

    #[test]
    fn detect_and_convert_unicode_points_multiple() {
        let input = "U+4F60 U+597D";
        assert_eq!(detect_type(input), InputType::UnicodePoints);
        let (out, _) = convert(input, &InputType::UnicodePoints);
        assert_eq!(out, "你好");
    }

    #[test]
    fn detect_and_convert_unicode_escaped() {
        let input = "\\u4f60\\u597d";
        assert_eq!(detect_type(input), InputType::UnicodeEscaped);
        let (out, _) = convert(input, &InputType::UnicodeEscaped);
        assert_eq!(out, "你好");
    }

    #[test]
    fn detect_and_convert_jwt_payload() {
        let header = general_purpose::URL_SAFE_NO_PAD.encode(r#"{"alg":"none"}"#);
        let payload = general_purpose::URL_SAFE_NO_PAD.encode(r#"{"sub":"123"}"#);
        let token = format!("{}.{}.{}", header, payload, "sig");

        assert_eq!(detect_type(&token), InputType::Jwt);
        let (out, info) = convert(&token, &InputType::Jwt);
        assert!(info.contains("JWT"));
        assert!(out.contains("\"sub\""));
        assert!(out.contains("123"));
    }
}
