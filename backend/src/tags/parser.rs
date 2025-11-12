use serde::{Deserialize, Serialize};

/// Normalized tag representation produced from filenames.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub raw_token: String,
    #[serde(rename = "type")]
    pub kind: TagKind,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    pub normalized: String,
}

/// Distinguishes between simple tags and key/value attributes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TagKind {
    Simple,
    KeyValue,
}

/// Result of parsing a filename into tags.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct TagParseResult {
    pub tags: Vec<Tag>,
    pub invalid_tokens: Vec<String>,
}

/// Parse the tag tokens from a filename (without directories).
///
/// Tokens are expected to use `_` or `+` as delimiters, with key/value pairs
/// represented as `key-value` (or `key:value`). Returns the parsed tags plus a
/// list of invalid tokens that were skipped.
pub fn parse_filename_tokens(filename: &str) -> TagParseResult {
    let stem = filename.split('.').next().unwrap_or(filename);
    let mut result = TagParseResult::default();

    for token in stem.split(|c: char| c == '_' || c == '+' || c.is_whitespace()) {
        let raw = token.trim();
        if raw.is_empty() {
            continue;
        }

        match classify_token(raw) {
            Some(TagParts::Simple { name }) => {
                result.tags.push(Tag {
                    raw_token: raw.to_string(),
                    kind: TagKind::Simple,
                    normalized: normalize_simple(&name),
                    name: normalize_simple(&name),
                    value: None,
                });
            }
            Some(TagParts::KeyValue { key, value }) => {
                let name = normalize_simple(&key);
                let normalized_value = normalize_simple(&value);
                let normalized = format!("{name}={normalized_value}");
                result.tags.push(Tag {
                    raw_token: raw.to_string(),
                    kind: TagKind::KeyValue,
                    name,
                    value: Some(normalized_value),
                    normalized,
                });
            }
            None => result.invalid_tokens.push(raw.to_string()),
        }
    }

    result
}

enum TagParts {
    Simple { name: String },
    KeyValue { key: String, value: String },
}

fn classify_token(token: &str) -> Option<TagParts> {
    if token.contains(':') || token.contains('-') {
        if let Some((key, value)) = split_kv(token, ':').or_else(|| split_kv(token, '-')) {
            return Some(TagParts::KeyValue { key, value });
        } else {
            return None;
        }
    }

    let normalized = token.trim();
    if normalized.is_empty() {
        None
    } else {
        Some(TagParts::Simple {
            name: normalized.to_string(),
        })
    }
}

fn split_kv(token: &str, delimiter: char) -> Option<(String, String)> {
    let idx = token.find(delimiter)?;
    let (key, rest) = token.split_at(idx);
    let value = &rest[1..];
    let key = key.trim();
    let value = value.trim();
    if key.is_empty() || value.is_empty() {
        return None;
    }
    Some((key.to_string(), value.to_string()))
}

fn normalize_simple(token: &str) -> String {
    token.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sample_filename_into_tags() {
        let result = parse_filename_tokens("sunset_coast+location-okinawa_rating-5");
        assert_eq!(result.invalid_tokens, Vec::<String>::new());
        assert_eq!(
            result
                .tags
                .iter()
                .map(|tag| tag.normalized.clone())
                .collect::<Vec<_>>(),
            vec![
                "sunset".to_string(),
                "coast".to_string(),
                "location=okinawa".to_string(),
                "rating=5".to_string()
            ]
        );
    }

    #[test]
    fn captures_invalid_tokens() {
        let result = parse_filename_tokens("invalid- rating-  _good+ :missing");
        assert_eq!(result.tags.len(), 1);
        assert_eq!(result.tags[0].normalized, "good");
        assert_eq!(
            result.invalid_tokens,
            vec!["invalid-", "rating-", ":missing"]
        );
    }
}
