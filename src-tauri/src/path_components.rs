use std::ops::Range;

const PREFIXES: &[&str] = &["natives/STM/", "natives/MSG/", "natives/NSW/"];
const LANGUAGES: &[&str] = &[
    "Ja", "En", "Fr", "It", "De", "Es", "Ru", "Pl", "Nl", "Pt", "PtBR", "Ko", "ZhTW", "ZhCN", "Fi",
    "Sv", "Da", "No", "Cs", "Hu", "Sk", "Ar", "Tr", "Bu", "Gr", "Ro", "Th", "Uk", "Vi", "Id", "Fc",
    "Hi", "Es419",
];

#[derive(Debug, Clone)]
pub struct PathComponents {
    normalized_full: String,
    raw_path: Range<usize>,
}

impl PathComponents {
    pub fn parse(line: &str) -> Option<Self> {
        let s = line.trim();
        if s.is_empty() || s.starts_with('#') {
            return None;
        }

        let normalized_full = if s.contains('\\') {
            s.replace('\\', "/")
        } else {
            s.to_string()
        };

        let (normalized_full, raw_path) = parse_raw_path_range(normalized_full);

        Some(Self {
            normalized_full,
            raw_path,
        })
    }

    pub fn raw_path(&self) -> &str {
        &self.normalized_full[self.raw_path.clone()]
    }

    #[cfg(test)]
    pub fn version_str(&self) -> Option<&str> {
        let dot = self.raw_path.end;
        if dot >= self.normalized_full.len() {
            return None;
        }
        if self.normalized_full.as_bytes().get(dot) != Some(&b'.') {
            return None;
        }

        let start = dot + 1;
        let end = self.normalized_full[start..]
            .find('.')
            .map(|rel| start + rel)
            .unwrap_or(self.normalized_full.len());

        let seg = &self.normalized_full[start..end];
        if is_digits(seg) { Some(seg) } else { None }
    }

    pub fn extension(&self) -> Option<&str> {
        let raw = self.raw_path();
        let dot = raw.rfind('.')?;
        Some(&raw[dot + 1..])
    }
}

fn last_segment_range(s: &str, end: usize) -> Option<(Range<usize>, usize)> {
    let dot = s.get(..end)?.rfind('.')?;
    Some(((dot + 1)..end, dot))
}

fn is_digits(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

fn is_tag(s: &str) -> bool {
    is_language_tag(s) || is_platform_tag(s) || is_arch_tag(s)
}

fn parse_raw_path_range(mut normalized_full: String) -> (String, Range<usize>) {
    while normalized_full.starts_with('@') || normalized_full.starts_with('/') {
        normalized_full.remove(0);
    }

    let mut raw_start = 0usize;
    for prefix in PREFIXES {
        if starts_with_ignore_ascii_case(normalized_full.as_str(), prefix) {
            raw_start = prefix.len();
            break;
        }
    }
    if raw_start == 0 {
        for prefix in PREFIXES {
            if let Some(pos) = find_ignore_ascii_case(normalized_full.as_str(), prefix) {
                normalized_full.drain(..pos);
                raw_start = prefix.len();
                break;
            }
        }
    }

    if let Some(rest) = strip_prefix_ignore_ascii_case(&normalized_full[raw_start..], "streaming/")
    {
        raw_start = normalized_full.len() - rest.len();
    }

    let mut raw_end = normalized_full.len();
    if let Some((seg_a, dot_a)) = last_segment_range(&normalized_full, normalized_full.len()) {
        let seg_a_str = &normalized_full[seg_a.clone()];

        if is_digits(seg_a_str) {
            raw_end = dot_a;
        } else if is_tag(seg_a_str)
            && let Some((seg_b, dot_b)) = last_segment_range(&normalized_full, dot_a)
        {
            let seg_b_str = &normalized_full[seg_b.clone()];
            if is_digits(seg_b_str) {
                raw_end = dot_b;
            } else if is_tag(seg_b_str)
                && let Some((seg_c, dot_c)) = last_segment_range(&normalized_full, dot_b)
            {
                let seg_c_str = &normalized_full[seg_c.clone()];
                if is_digits(seg_c_str) {
                    raw_end = dot_c;
                }
            }
        }
    }

    if raw_end < raw_start {
        raw_end = raw_start;
    }

    (normalized_full, raw_start..raw_end)
}

fn is_platform_tag(s: &str) -> bool {
    s.eq_ignore_ascii_case("STM") || s.eq_ignore_ascii_case("NSW") || s.eq_ignore_ascii_case("MSG")
}

fn is_arch_tag(s: &str) -> bool {
    s.eq_ignore_ascii_case("X64")
}

fn is_language_tag(s: &str) -> bool {
    LANGUAGES.iter().any(|lang| lang.eq_ignore_ascii_case(s))
}

fn starts_with_ignore_ascii_case(s: &str, prefix: &str) -> bool {
    s.get(..prefix.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
}

fn strip_prefix_ignore_ascii_case<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    let head = s.get(..prefix.len())?;
    if head.eq_ignore_ascii_case(prefix) {
        Some(&s[prefix.len()..])
    } else {
        None
    }
}

fn find_ignore_ascii_case(haystack: &str, needle: &str) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    let hay = haystack.as_bytes();
    let ned = needle.as_bytes();
    if ned.len() > hay.len() {
        return None;
    }
    hay.windows(ned.len()).position(|window| {
        window
            .iter()
            .zip(ned.iter())
            .all(|(&a, &b)| a.eq_ignore_ascii_case(&b))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_keeps_raw_path_without_version() {
        let path = PathComponents::parse(
            "natives/stm/systems/rendering/bluenoise256x256/hdr_rgba_0028.tex.251111100",
        )
        .unwrap();

        assert_eq!(
            path.raw_path(),
            "systems/rendering/bluenoise256x256/hdr_rgba_0028.tex"
        );
        assert_eq!(path.version_str(), Some("251111100"));
        assert_eq!(path.extension(), Some("tex"));
    }

    #[test]
    fn test_parse_keeps_raw_path_without_version_tags() {
        let path = PathComponents::parse("foo.tex.241106027.X64").unwrap();

        assert_eq!(path.raw_path(), "foo.tex");
        assert_eq!(path.version_str(), Some("241106027"));
        assert_eq!(path.extension(), Some("tex"));
    }
}
