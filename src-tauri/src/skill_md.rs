/// SKILL.md / AGENTS.md 前字段解析与依赖检测

pub fn extract_field(content: &str, field: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    for line in lines {
        if line.contains(field) {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let value = parts[1].trim().trim_matches('"').trim_matches('"');
                return Some(value.to_string());
            }
        }
    }
    None
}

pub fn extract_array(content: &str, field: &str) -> Vec<String> {
    let mut result = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut in_array = false;

    for line in lines {
        if line.contains(field) && line.contains('[') {
            in_array = true;
            if let Some(start) = line.find('[') {
                if let Some(end) = line.find(']') {
                    let arr_str = &line[start + 1..end];
                    result = arr_str
                        .split(',')
                        .map(|s| s.trim().trim_matches('"').to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    in_array = false;
                }
            }
        } else if in_array {
            if line.contains(']') {
                break;
            }
            let value = line.trim().trim_matches(',').trim_matches('"').to_string();
            if !value.is_empty() {
                result.push(value);
            }
        }
    }

    result
}

pub fn bin_present(bin: &str) -> bool {
    std::process::Command::new("which")
        .arg(bin)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// `bins` 中在 PATH 里找不到可执行文件的项（与 `ready` 判定一致：`requires.is_empty() || missing.is_empty()`）
pub fn missing_bins_list(bins: &[String]) -> Vec<String> {
    bins
        .iter()
        .filter(|b| !bin_present(b))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_field_trims_quotes() {
        let md = r#"name: "openhue"
description: Light control
"#;
        assert_eq!(extract_field(md, "name"), Some("openhue".into()));
    }

    #[test]
    fn extract_array_bins_single_line() {
        let md = "bins: [rg, fd, jq]";
        assert_eq!(
            extract_array(md, "bins"),
            vec!["rg".to_string(), "fd".to_string(), "jq".to_string()]
        );
    }

    #[test]
    fn extract_array_multiline_bins() {
        let md = r#"bins: [
  ffmpeg,
  imagemagick,
]"#;
        let v = extract_array(md, "bins");
        assert!(v.contains(&"ffmpeg".to_string()));
        assert!(v.contains(&"imagemagick".to_string()));
    }
}
