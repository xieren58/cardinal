use query_segmentation::Segment;
use regex::{Regex, RegexBuilder};

#[derive(Debug, Clone, Copy, Default)]
pub struct SearchOptions {
    pub case_insensitive: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum SegmentKind {
    Substr,
    Prefix,
    Suffix,
    Exact,
}

#[derive(Clone, Debug)]
pub(crate) enum SegmentMatcher {
    Plain { kind: SegmentKind, needle: String },
    Regex { regex: Regex },
}

impl SegmentMatcher {
    pub(crate) fn matches(&self, candidate: &str) -> bool {
        match self {
            SegmentMatcher::Plain { kind, needle } => match kind {
                SegmentKind::Substr => candidate.contains(needle),
                SegmentKind::Prefix => candidate.starts_with(needle),
                SegmentKind::Suffix => candidate.ends_with(needle),
                SegmentKind::Exact => candidate == needle,
            },
            SegmentMatcher::Regex { regex } => regex.is_match(candidate),
        }
    }
}

fn wildcard_to_regex(pattern: &str) -> String {
    let mut regex = String::with_capacity(pattern.len() + 3);
    regex.push('^');
    for ch in pattern.chars() {
        match ch {
            '*' => regex.push_str(".*"),
            '?' => regex.push('.'),
            _ => {
                let mut buf = [0u8; 4];
                let encoded = ch.encode_utf8(&mut buf);
                regex.push_str(&regex::escape(encoded));
            }
        }
    }
    regex.push('$');
    regex
}

pub(crate) fn build_segment_matchers(
    segments: &[Segment<'_>],
    options: SearchOptions,
) -> Result<Vec<SegmentMatcher>, regex::Error> {
    segments
        .iter()
        .map(|segment| {
            let kind = segment_kind(segment);
            let value = segment_value(segment);
            let is_wildcard = value.contains("*") || value.contains('?');
            if options.case_insensitive || is_wildcard {
                let pattern = if is_wildcard {
                    // Wildcard pattern is /exact/ by default, so we don't need to
                    // adjust it based on SegmentKind.
                    wildcard_to_regex(value)
                } else {
                    let base = regex::escape(value);
                    match kind {
                        SegmentKind::Substr => base,
                        SegmentKind::Prefix => format!("^(?:{base})"),
                        SegmentKind::Suffix => format!("(?:{base})$"),
                        SegmentKind::Exact => format!("^(?:{base})$"),
                    }
                };
                let mut builder = RegexBuilder::new(&pattern);
                builder.case_insensitive(options.case_insensitive);
                builder.build().map(|regex| SegmentMatcher::Regex { regex })
            } else {
                Ok(SegmentMatcher::Plain {
                    kind,
                    needle: value.to_string(),
                })
            }
        })
        .collect()
}

fn segment_kind(segment: &Segment<'_>) -> SegmentKind {
    match segment {
        Segment::Substr(_) => SegmentKind::Substr,
        Segment::Prefix(_) => SegmentKind::Prefix,
        Segment::Suffix(_) => SegmentKind::Suffix,
        Segment::Exact(_) => SegmentKind::Exact,
    }
}

fn segment_value<'s>(segment: &Segment<'s>) -> &'s str {
    match segment {
        Segment::Substr(value)
        | Segment::Prefix(value)
        | Segment::Suffix(value)
        | Segment::Exact(value) => value,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SearchOptions, SegmentKind, SegmentMatcher, build_segment_matchers, segment_kind,
        segment_value, wildcard_to_regex,
    };
    use query_segmentation::Segment;

    // --- wildcard_to_regex edge cases ---

    #[test]
    fn wildcard_glob_tokens_are_converted() {
        assert_eq!(wildcard_to_regex("foo*bar?baz"), "^foo.*bar.baz$");
    }

    #[test]
    fn wildcard_only_star() {
        assert_eq!(wildcard_to_regex("*"), "^.*$");
    }

    #[test]
    fn wildcard_only_question() {
        assert_eq!(wildcard_to_regex("?"), "^.$");
    }

    #[test]
    fn wildcard_mixed_starts_ends() {
        assert_eq!(wildcard_to_regex("*foo?"), "^.*foo.$");
        assert_eq!(wildcard_to_regex("?foo*"), "^.foo.*$");
    }

    #[test]
    fn wildcard_escapes_regex_characters() {
        assert_eq!(wildcard_to_regex("file.+(1)"), "^file\\.\\+\\(1\\)$");
    }

    #[test]
    fn wildcard_escapes_unicode() {
        // '?' acts as a wildcard (single character), not a literal, so it becomes '.' in the regex.
        assert_eq!(wildcard_to_regex("café*(a?)"), "^café.*\\(a.\\)$");
    }

    #[test]
    fn wildcard_empty_string() {
        assert_eq!(wildcard_to_regex(""), "^$");
    }

    // --- segment_kind mapping ---

    #[test]
    fn segment_kind_mapping() {
        assert!(matches!(
            segment_kind(&Segment::Substr("abc")),
            SegmentKind::Substr
        ));
        assert!(matches!(
            segment_kind(&Segment::Prefix("abc")),
            SegmentKind::Prefix
        ));
        assert!(matches!(
            segment_kind(&Segment::Suffix("abc")),
            SegmentKind::Suffix
        ));
        assert!(matches!(
            segment_kind(&Segment::Exact("abc")),
            SegmentKind::Exact
        ));
    }

    #[test]
    fn segment_value_extraction() {
        assert_eq!(segment_value(&Segment::Substr("abc")), "abc");
        assert_eq!(segment_value(&Segment::Prefix("def")), "def");
        assert_eq!(segment_value(&Segment::Suffix("ghi")), "ghi");
        assert_eq!(segment_value(&Segment::Exact("jkl")), "jkl");
    }

    // --- build_segment_matchers (plain, no wildcard, case-sensitive) ---

    #[test]
    fn build_plain_matchers_without_case_insensitive() {
        let segments = [
            Segment::Substr("mid"),
            Segment::Prefix("pre"),
            Segment::Suffix("suf"),
            Segment::Exact("exact"),
        ];
        let opts = SearchOptions {
            case_insensitive: false,
        };
        let matchers = build_segment_matchers(&segments, opts).expect("ok");
        assert_eq!(matchers.len(), 4);
        // All should be Plain
        for (m, s) in matchers.iter().zip(segments.iter()) {
            match m {
                SegmentMatcher::Plain { kind, needle } => {
                    assert_eq!(needle, segment_value(s));
                    assert_eq!(*kind as u8, segment_kind(s) as u8);
                }
                _ => panic!("Expected Plain matcher"),
            }
        }
    }

    // --- build_segment_matchers with case_insensitive true (regex) ---

    #[test]
    fn build_regex_matchers_case_insensitive() {
        let segments = [
            Segment::Substr("mid"),
            Segment::Prefix("pre"),
            Segment::Suffix("suf"),
            Segment::Exact("exact"),
        ];
        let opts = SearchOptions {
            case_insensitive: true,
        };
        let matchers = build_segment_matchers(&segments, opts).expect("ok");
        assert_eq!(matchers.len(), 4);
        let patterns: Vec<_> = matchers
            .iter()
            .map(|m| match m {
                SegmentMatcher::Regex { regex } => regex.as_str().to_string(),
                _ => panic!("Expected Regex matcher"),
            })
            .collect();
        assert_eq!(patterns[0], "mid"); // substr
        assert_eq!(patterns[1], "^(?:pre)"); // prefix
        assert_eq!(patterns[2], "(?:suf)$"); // suffix
        assert_eq!(patterns[3], "^(?:exact)$"); // exact
    }

    // --- wildcard forces regex even when case_sensitive ---

    #[test]
    fn wildcard_forces_regex_exact_anchor() {
        let segments = [Segment::Exact("foo*bar?baz")];
        let opts = SearchOptions {
            case_insensitive: false,
        };
        let matchers = build_segment_matchers(&segments, opts).expect("ok");
        assert_eq!(matchers.len(), 1);
        match &matchers[0] {
            SegmentMatcher::Regex { regex } => {
                assert_eq!(regex.as_str(), "^foo.*bar.baz$");
            }
            _ => panic!("Expected regex for wildcard segment"),
        }
    }

    #[test]
    fn wildcard_case_sensitive() {
        let segments = [Segment::Substr("A*B")];
        let opts = SearchOptions {
            case_insensitive: false,
        };
        let matchers = build_segment_matchers(&segments, opts).expect("ok");
        match &matchers[0] {
            SegmentMatcher::Regex { regex } => {
                assert!(!regex.is_match("aXXb"));
                assert!(regex.is_match("AXXB"));
            }
            _ => panic!("Expected regex matcher"),
        }
    }

    #[test]
    fn wildcard_case_insensitive() {
        let segments = [Segment::Substr("A*B")];
        let opts = SearchOptions {
            case_insensitive: true,
        };
        let matchers = build_segment_matchers(&segments, opts).expect("ok");
        match &matchers[0] {
            SegmentMatcher::Regex { regex } => {
                assert!(regex.is_match("aXXb"));
                assert!(regex.is_match("AXXB"));
            }
            _ => panic!("Expected regex matcher"),
        }
    }

    // --- SegmentMatcher.matches for Plain variants ---

    #[test]
    fn plain_substr_matches() {
        let m = SegmentMatcher::Plain {
            kind: SegmentKind::Substr,
            needle: "abc".into(),
        };
        assert!(m.matches("zzzabczzz"));
        assert!(!m.matches("abz"));
    }

    #[test]
    fn plain_prefix_matches() {
        let m = SegmentMatcher::Plain {
            kind: SegmentKind::Prefix,
            needle: "start".into(),
        };
        assert!(m.matches("start_of_line"));
        assert!(!m.matches("line_start"));
    }

    #[test]
    fn plain_suffix_matches() {
        let m = SegmentMatcher::Plain {
            kind: SegmentKind::Suffix,
            needle: "tail".into(),
        };
        assert!(m.matches("segment_tail"));
        assert!(!m.matches("tail_segment"));
    }

    #[test]
    fn plain_exact_matches() {
        let m = SegmentMatcher::Plain {
            kind: SegmentKind::Exact,
            needle: "only".into(),
        };
        assert!(m.matches("only"));
        assert!(!m.matches("only1"));
    }

    // --- SegmentMatcher.matches for Regex variants ---

    #[test]
    fn regex_substr_equivalent() {
        let segments = [Segment::Substr("abc")];
        let opts = SearchOptions {
            case_insensitive: true,
        };
        let m = build_segment_matchers(&segments, opts).unwrap().remove(0);
        match m {
            SegmentMatcher::Regex { regex } => {
                assert!(regex.is_match("zzzAbCzzz"));
            }
            _ => panic!("Expected regex matcher"),
        }
    }

    #[test]
    fn regex_prefix_equivalent() {
        let segments = [Segment::Prefix("abc")];
        let opts = SearchOptions {
            case_insensitive: true,
        };
        let m = build_segment_matchers(&segments, opts).unwrap().remove(0);
        match m {
            SegmentMatcher::Regex { regex } => {
                assert!(regex.is_match("AbCzzz"));
                assert!(!regex.is_match("zzzabc"));
            }
            _ => panic!("Expected regex matcher"),
        }
    }

    #[test]
    fn regex_suffix_equivalent() {
        let segments = [Segment::Suffix("abc")];
        let opts = SearchOptions {
            case_insensitive: true,
        };
        let m = build_segment_matchers(&segments, opts).unwrap().remove(0);
        match m {
            SegmentMatcher::Regex { regex } => {
                assert!(regex.is_match("zzzAbC"));
                assert!(!regex.is_match("AbCzzz"));
            }
            _ => panic!("Expected regex matcher"),
        }
    }

    #[test]
    fn regex_exact_equivalent() {
        let segments = [Segment::Exact("abc")];
        let opts = SearchOptions {
            case_insensitive: true,
        };
        let m = build_segment_matchers(&segments, opts).unwrap().remove(0);
        match m {
            SegmentMatcher::Regex { regex } => {
                assert!(regex.is_match("AbC"));
                assert!(!regex.is_match("xabc"));
            }
            _ => panic!("Expected regex matcher"),
        }
    }

    // --- Mixed segments producing both Plain and Regex ---

    #[test]
    fn mixed_segments_plain_and_regex() {
        let segments = [
            Segment::Substr("abc"),   // plain
            Segment::Prefix("pre"),   // plain
            Segment::Suffix("*wild"), // wildcard => regex
            Segment::Exact("ex?act"), // wildcard => regex
        ];
        let opts = SearchOptions {
            case_insensitive: false,
        };
        let matchers = build_segment_matchers(&segments, opts).unwrap();
        assert_eq!(matchers.len(), 4);
        assert!(matches!(matchers[0], SegmentMatcher::Plain { .. }));
        assert!(matches!(matchers[1], SegmentMatcher::Plain { .. }));
        assert!(matches!(matchers[2], SegmentMatcher::Regex { .. }));
        assert!(matches!(matchers[3], SegmentMatcher::Regex { .. }));
    }

    #[test]
    fn mixed_segments_all_regex_when_case_insensitive() {
        let segments = [
            Segment::Substr("abc"),
            Segment::Prefix("pre"),
            Segment::Suffix("suf"),
            Segment::Exact("exact"),
        ];
        let opts = SearchOptions {
            case_insensitive: true,
        };
        let matchers = build_segment_matchers(&segments, opts).unwrap();
        for m in matchers {
            assert!(matches!(m, SegmentMatcher::Regex { .. }));
        }
    }

    // --- Wildcard escaping ensures metacharacters are literal ---

    #[test]
    fn wildcard_metacharacters_literal() {
        let segments = [Segment::Exact("a+b*(c?)")];
        let opts = SearchOptions {
            case_insensitive: false,
        };
        let matchers = build_segment_matchers(&segments, opts).unwrap();
        match &matchers[0] {
            SegmentMatcher::Regex { regex } => {
                // '?' is treated as wildcard -> '.'
                assert_eq!(regex.as_str(), "^a\\+b.*\\(c.\\)$");
                assert!(regex.is_match("a+bZZZ(c?)"));
                assert!(!regex.is_match("abZZZ(c?)"));
            }
            _ => panic!("Expected regex"),
        }
    }

    // --- Unicode handling inside regex (no case folding when insensitive=false) ---

    #[test]
    fn unicode_case_sensitive() {
        let segments = [Segment::Substr("Café")];
        let opts = SearchOptions {
            case_insensitive: false,
        };
        let matchers = build_segment_matchers(&segments, opts).unwrap();
        match &matchers[0] {
            SegmentMatcher::Plain { needle, .. } => {
                assert_eq!(needle, "Café");
            }
            _ => panic!("Expected plain matcher"),
        }
    }

    #[test]
    fn unicode_case_insensitive() {
        let segments = [Segment::Exact("Café")];
        let opts = SearchOptions {
            case_insensitive: true,
        };
        let matchers = build_segment_matchers(&segments, opts).unwrap();
        match &matchers[0] {
            SegmentMatcher::Regex { regex } => {
                assert!(regex.is_match("café"));
                // Basic ASCII case fold works; regex crate may not fold é to É on all platforms, so we only check lowercase.
            }
            _ => panic!("Expected regex matcher"),
        }
    }

    // --- Ensure anchoring semantics for prefix/suffix/exact patterns ---

    #[test]
    fn anchoring_prefix_suffix_exact_patterns() {
        let segments = [
            Segment::Prefix("pre"),
            Segment::Suffix("suf"),
            Segment::Exact("all"),
        ];
        let opts = SearchOptions {
            case_insensitive: true,
        };
        let matchers = build_segment_matchers(&segments, opts).unwrap();
        assert_eq!(matchers.len(), 3);
        match &matchers[0] {
            SegmentMatcher::Regex { regex } => assert!(regex.as_str().starts_with("^(?:")),
            _ => panic!("regex expected"),
        }
        match &matchers[1] {
            SegmentMatcher::Regex { regex } => assert!(regex.as_str().ends_with(")$")),
            _ => panic!("regex expected"),
        }
        match &matchers[2] {
            SegmentMatcher::Regex { regex } => {
                assert!(regex.as_str().starts_with("^(?:"));
                assert!(regex.as_str().ends_with(")$"));
            }
            _ => panic!("regex expected"),
        }
    }

    // --- Large pattern capacity (sanity, does not panic) ---

    #[test]
    fn large_pattern_build() {
        let long = "a".repeat(10_000);
        let segments = [Segment::Exact(&long)];
        let opts = SearchOptions {
            case_insensitive: true,
        };
        let matchers = build_segment_matchers(&segments, opts).unwrap();
        assert_eq!(matchers.len(), 1);
        match &matchers[0] {
            SegmentMatcher::Regex { regex } => {
                assert!(regex.as_str().starts_with("^(?:"));
            }
            _ => panic!("Expected regex"),
        }
    }

    // --- Multiple wildcards in one segment ---

    #[test]
    fn multiple_wildcards_match() {
        let segments = [Segment::Exact("a*b*c?d")];
        let opts = SearchOptions {
            case_insensitive: false,
        };
        let matchers = build_segment_matchers(&segments, opts).unwrap();
        match &matchers[0] {
            SegmentMatcher::Regex { regex } => {
                assert!(regex.is_match("aZZbYYcXd"));
                assert!(!regex.is_match("abYcXdX"));
            }
            _ => panic!("Expected regex"),
        }
    }

    // --- Ensure plain Substr acts like contains ---

    #[test]
    fn substr_plain_contains_behavior() {
        let segments = [Segment::Substr("mid")];
        let opts = SearchOptions {
            case_insensitive: false,
        };
        let matchers = build_segment_matchers(&segments, opts).unwrap();
        match &matchers[0] {
            SegmentMatcher::Plain { needle, .. } => {
                assert_eq!(needle, "mid");
                assert!(
                    SegmentMatcher::Plain {
                        kind: SegmentKind::Substr,
                        needle: needle.clone()
                    }
                    .matches("xxmidxx")
                );
            }
            _ => panic!("Expected plain matcher"),
        }
    }
}
