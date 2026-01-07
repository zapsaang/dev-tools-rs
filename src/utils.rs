use serde::Serialize;
use std::io::{self, Read};

// --- Alfred JSON 结构 ---
#[derive(Serialize)]
pub struct AlfredOutput {
    pub items: Vec<AlfredItem>,
}

#[derive(Serialize)]
pub struct AlfredItem {
    pub uid: String,
    pub title: String,
    pub subtitle: String,
    pub arg: String,
    pub text: AlfredText,
    pub icon: AlfredIcon,
}

#[derive(Serialize)]
pub struct AlfredText {
    pub copy: String,
    pub largetype: String,
}

#[derive(Serialize)]
pub struct AlfredIcon {
    #[serde(rename = "type")]
    pub icon_type: String,
}

impl AlfredItem {
    pub fn new(uid: &str, title: &str, subtitle: &str, arg: &str) -> Self {
        Self {
            uid: uid.to_string(),
            title: title.to_string(),
            subtitle: subtitle.to_string(),
            arg: arg.to_string(),
            text: AlfredText {
                copy: arg.to_string(),
                largetype: arg.to_string(),
            },
            icon: AlfredIcon {
                icon_type: "default".to_string(),
            },
        }
    }
}

pub fn print_alfred(items: Vec<AlfredItem>) {
    let output = AlfredOutput { items };
    println!("{}", serde_json::to_string(&output).unwrap());
}

// --- 输入读取 ---
pub fn read_input(args: Option<Vec<String>>) -> String {
    match args {
        Some(v) if !v.is_empty() => v.join(" "),
        _ => {
            // 如果不是交互式终端，尝试从 Pipe 读取
            if !atty::is(atty::Stream::Stdin) {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer).unwrap_or(0);
                buffer.trim().to_string()
            } else {
                String::new()
            }
        }
    }
}

// 判断是否应该进入 Alfred 模式
pub fn is_alfred_env(flag: bool) -> bool {
    flag || (!atty::is(atty::Stream::Stdout))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alfred_item_new_sets_copy_fields() {
        let item = AlfredItem::new("id", "title", "subtitle", "arg");
        assert_eq!(item.uid, "id");
        assert_eq!(item.title, "title");
        assert_eq!(item.subtitle, "subtitle");
        assert_eq!(item.arg, "arg");
        assert_eq!(item.text.copy, "arg");
        assert_eq!(item.text.largetype, "arg");
    }
}