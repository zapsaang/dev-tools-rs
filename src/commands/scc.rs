use crate::utils::{AlfredItem, is_alfred_env, print_alfred, read_input};
use clap::{Args, ValueEnum};
use rand::Rng;

#[derive(Args)]
pub struct SccArgs {
    /// 输入字符串
    #[arg(value_name = "INPUT")]
    input: Option<Vec<String>>,

    /// 指定目标格式
    #[arg(short, long, value_enum)]
    format: Option<SccFormat>,

    /// 强制 Alfred JSON 输出
    #[arg(long)]
    alfred: bool,

    /// 列出所有可用格式
    #[arg(long)]
    list: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum SccFormat {
    Camel,
    Pascal,
    Snake,
    SnakeUpper,
    SnakeCap,
    Kebab,
    Dot,
    Title,
    SpongeBob,
    Upper,
    Lower,
}

fn normalize(input: &str) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    let mut current = String::new();

    let mut prev: Option<char> = None;

    for c in input.chars() {
        if !c.is_ascii_alphanumeric() {
            if !current.is_empty() {
                words.push(current.to_lowercase());
                current.clear();
            }
            prev = None;
            continue;
        }

        if current.is_empty() {
            current.push(c);
            prev = Some(c);
            continue;
        }

        let p = prev.unwrap();
        let boundary = (p.is_ascii_lowercase() && c.is_ascii_uppercase())
            || (p.is_ascii_alphabetic() && c.is_ascii_digit())
            || (p.is_ascii_digit() && c.is_ascii_alphabetic());

        // 处理缩写："HTTPServer" => ["HTTP", "Server"]
        // 当出现 "...PS" + "e" 这种 Upper->Lower 过渡时，把最后一个 Upper 移到新词里。
        let acronym_boundary = p.is_ascii_uppercase()
            && c.is_ascii_lowercase()
            && current.len() >= 2
            && current
                .chars()
                .rev()
                .nth(1)
                .is_some_and(|pp| pp.is_ascii_uppercase());

        if acronym_boundary {
            let last = current.pop().unwrap();
            if !current.is_empty() {
                words.push(current.to_lowercase());
            }
            current.clear();
            current.push(last);
            current.push(c);
        } else if boundary {
            words.push(current.to_lowercase());
            current.clear();
            current.push(c);
        } else {
            current.push(c);
        }

        prev = Some(c);
    }

    if !current.is_empty() {
        words.push(current.to_lowercase());
    }

    words
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

// 嘲讽海绵宝宝风格: mOcKiNg sPoNgEbOb
fn to_spongebob(original: &str) -> String {
    let mut rng = rand::thread_rng();
    original
        .chars()
        .map(|c| {
            if c.is_alphabetic() {
                if rng.gen_bool(0.5) {
                    c.to_uppercase().to_string()
                } else {
                    c.to_lowercase().to_string()
                }
            } else {
                c.to_string()
            }
        })
        .collect()
}

fn convert(words: &[String], mode: SccFormat, original: &str) -> String {
    match mode {
        SccFormat::Camel => words
            .iter()
            .enumerate()
            .map(|(i, w)| if i == 0 { w.clone() } else { capitalize(w) })
            .collect(),
        SccFormat::Pascal => words.iter().map(|w| capitalize(w)).collect(),
        SccFormat::Snake => words.join("_"),
        SccFormat::SnakeUpper => words.join("_").to_uppercase(),
        SccFormat::SnakeCap => words
            .iter()
            .map(|w| capitalize(w))
            .collect::<Vec<_>>()
            .join("_"),
        SccFormat::Kebab => words.join("-"),
        SccFormat::Dot => words.join("."),
        SccFormat::Title => words
            .iter()
            .map(|w| capitalize(w))
            .collect::<Vec<_>>()
            .join(" "), // Hello World
        SccFormat::SpongeBob => to_spongebob(original),
        SccFormat::Upper => original.to_uppercase(),
        SccFormat::Lower => original.to_lowercase(),
    }
}

pub fn run(args: SccArgs) {
    if args.list {
        println!(
            "camel\npascal\nsnake\nsnake_upper\nsnake_cap\nkebab\ndot\ntitle\nspongebob\nupper\nlower"
        );
        return;
    }

    let input_str = read_input(args.input);
    if input_str.is_empty() {
        return;
    }

    let is_alfred = is_alfred_env(args.alfred) && args.format.is_none();
    let words = normalize(&input_str);

    if !is_alfred {
        if let Some(fmt) = args.format {
            print!("{}", convert(&words, fmt, &input_str));
        } else {
            eprintln!("Error: format required in non-alfred mode (use -f)");
        }
    } else {
        let styles = vec![
            (SccFormat::Camel, "camelCase | 小驼峰"),
            (SccFormat::Pascal, "PascalCase | 大驼峰"),
            (SccFormat::Snake, "snake_case | 小蛇形"),
            (SccFormat::SnakeUpper, "SNAKE_CASE | 大蛇形"),
            (SccFormat::SnakeCap, "Snake_Case | 首字母大写蛇形"),
            (SccFormat::Kebab, "kebab-case | 短横线"),
            (SccFormat::Dot, "dot.case | 点分隔"),
            (SccFormat::Title, "Title Case | 标题"),
            (SccFormat::Upper, "UPPER | 全大写"),
            (SccFormat::Lower, "lower | 全小写"),
            (SccFormat::SpongeBob, "mOcKiNg | 嘲讽模式"),
        ];

        let items: Vec<AlfredItem> = styles
            .into_iter()
            .map(|(fmt, label)| {
                let val = convert(&words, fmt, &input_str);
                AlfredItem::new("scc", &val, label, &val)
            })
            .collect();

        print_alfred(items);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_splits_common_patterns() {
        assert_eq!(normalize("helloWorld"), vec!["hello", "world"]);
        assert_eq!(normalize("HelloWorld"), vec!["hello", "world"]);
        assert_eq!(normalize("hello_world"), vec!["hello", "world"]);
        assert_eq!(normalize("hello-world"), vec!["hello", "world"]);
        assert_eq!(normalize("HTTPServer2"), vec!["http", "server", "2"]);
        assert_eq!(normalize("foo2bar"), vec!["foo", "2", "bar"]);
    }

    #[test]
    fn convert_basic_formats() {
        let words = normalize("helloWorld");
        assert_eq!(
            convert(&words, SccFormat::Snake, "helloWorld"),
            "hello_world"
        );
        assert_eq!(
            convert(&words, SccFormat::Kebab, "helloWorld"),
            "hello-world"
        );
        assert_eq!(
            convert(&words, SccFormat::Camel, "helloWorld"),
            "helloWorld"
        );
        assert_eq!(
            convert(&words, SccFormat::Pascal, "helloWorld"),
            "HelloWorld"
        );
        assert_eq!(
            convert(&words, SccFormat::Title, "helloWorld"),
            "Hello World"
        );
    }

    #[test]
    fn spongebob_keeps_non_alphabetic_chars() {
        let original = "a-b_c 1";
        let words = normalize(original);
        let out = convert(&words, SccFormat::SpongeBob, original);
        assert_eq!(out.len(), original.len());
        for (o, c) in original.chars().zip(out.chars()) {
            if !o.is_alphabetic() {
                assert_eq!(o, c);
            }
        }
    }
}
