use std::collections::{HashMap, HashSet};

use crate::{cache::CacheSnapshot, indexer::MediaFile, tags::TagKind};

const DEFAULT_PAGE_SIZE: usize = 60;
const MAX_PAGE_SIZE: usize = 200;

/// Normalized search input used by the backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery {
    required_tags: Vec<String>,
    attribute_filters: HashMap<String, HashSet<String>>,
    page: usize,
    page_size: usize,
}

impl SearchQuery {
    pub fn new(
        tags: Vec<String>,
        attributes: HashMap<String, Vec<String>>,
        page: usize,
        page_size: usize,
    ) -> Self {
        let required_tags = tags.into_iter().filter_map(normalize_token).collect();

        let attribute_filters = attributes
            .into_iter()
            .filter_map(|(key, values)| {
                let key = normalize_token(key)?;
                let value_set: HashSet<String> =
                    values.into_iter().filter_map(normalize_token).collect();
                if value_set.is_empty() {
                    None
                } else {
                    Some((key, value_set))
                }
            })
            .collect();

        Self {
            required_tags,
            attribute_filters,
            page: normalize_page(page),
            page_size: normalize_page_size(page_size),
        }
    }

    pub fn required_tags(&self) -> &[String] {
        &self.required_tags
    }

    pub fn attribute_filters(&self) -> &HashMap<String, HashSet<String>> {
        &self.attribute_filters
    }

    pub fn page(&self) -> usize {
        self.page
    }

    pub fn page_size(&self) -> usize {
        self.page_size
    }
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            required_tags: Vec::new(),
            attribute_filters: HashMap::new(),
            page: 1,
            page_size: DEFAULT_PAGE_SIZE,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub items: Vec<MediaFile>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

pub struct SearchService;

impl SearchService {
    pub fn execute(snapshot: &CacheSnapshot, query: &SearchQuery) -> SearchResult {
        let start_index = (query.page().saturating_sub(1)) * query.page_size();
        let mut collected = Vec::with_capacity(query.page_size());
        let mut matched_total = 0usize;

        for media in &snapshot.media {
            if !matches_required_tags(media, query.required_tags()) {
                continue;
            }
            if !matches_attributes(media, query.attribute_filters()) {
                continue;
            }

            if matched_total >= start_index && collected.len() < query.page_size() {
                collected.push(media.clone());
            }
            matched_total += 1;
        }

        SearchResult {
            items: collected,
            total: matched_total,
            page: query.page(),
            page_size: query.page_size(),
        }
    }
}

fn matches_required_tags(media: &MediaFile, required_tags: &[String]) -> bool {
    if required_tags.is_empty() {
        return true;
    }
    let tag_set: HashSet<&str> = media
        .tags
        .iter()
        .map(|tag| tag.normalized.as_str())
        .collect();
    required_tags
        .iter()
        .all(|tag| tag_set.contains(tag.as_str()))
}

fn matches_attributes(media: &MediaFile, filters: &HashMap<String, HashSet<String>>) -> bool {
    if filters.is_empty() {
        return true;
    }

    for (key, allowed_values) in filters {
        let mut matched = false;

        if let Some(value) = media.attributes.get(key) {
            if allowed_values.contains(&value.to_lowercase()) {
                matched = true;
            }
        }

        if !matched {
            matched = media
                .tags
                .iter()
                .filter(|tag| matches!(tag.kind, TagKind::KeyValue) && tag.name == *key)
                .filter_map(|tag| tag.value.as_ref())
                .any(|value| allowed_values.contains(value));
        }

        if !matched {
            return false;
        }
    }

    true
}

fn normalize_token<S: AsRef<str>>(token: S) -> Option<String> {
    let trimmed = token.as_ref().trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_lowercase())
    }
}

fn normalize_page(page: usize) -> usize {
    if page == 0 { 1 } else { page }
}

fn normalize_page_size(page_size: usize) -> usize {
    if page_size == 0 {
        DEFAULT_PAGE_SIZE
    } else {
        page_size.min(MAX_PAGE_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cache::CacheSnapshot,
        indexer::MediaType,
        tags::{Tag, TagKind},
    };
    use chrono::Utc;

    #[test]
    fn filters_by_tags_and_attributes() {
        let snapshot = fixture_snapshot();
        let mut attributes = HashMap::new();
        attributes.insert("rating".into(), vec!["5".into()]);
        let query = SearchQuery::new(vec!["sunset".into(), "coast".into()], attributes, 1, 10);
        let result = SearchService::execute(&snapshot, &query);
        assert_eq!(result.total, 1);
        assert_eq!(result.items[0].id, "sunset_A");
    }

    #[test]
    fn applies_or_semantics_within_attribute_values() {
        let snapshot = fixture_snapshot();
        let mut attributes = HashMap::new();
        attributes.insert("rating".into(), vec!["4".into(), "3".into()]);
        let query = SearchQuery::new(Vec::new(), attributes, 1, 10);
        let result = SearchService::execute(&snapshot, &query);
        assert_eq!(result.total, 3);
        let ids: HashSet<_> = result.items.iter().map(|m| m.id.as_str()).collect();
        assert!(ids.contains("macro_B"));
        assert!(ids.contains("video_C"));
        assert!(ids.contains("sunset_B"));
    }

    #[test]
    fn paginates_matches() {
        let snapshot = fixture_snapshot();
        let query = SearchQuery::new(vec!["sunset".into()], HashMap::new(), 2, 1);
        let result = SearchService::execute(&snapshot, &query);
        assert_eq!(result.total, 2);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].id, "sunset_B");
    }

    fn fixture_snapshot() -> CacheSnapshot {
        CacheSnapshot::new(vec![
            media(
                "sunset_A",
                vec![
                    simple_tag("sunset"),
                    simple_tag("coast"),
                    kv_tag("rating", "5"),
                ],
            ),
            media(
                "sunset_B",
                vec![simple_tag("sunset"), kv_tag("rating", "4")],
            ),
            media(
                "macro_B",
                vec![
                    simple_tag("macro"),
                    kv_tag("rating", "4"),
                    kv_tag("subject", "leaf"),
                ],
            ),
            media(
                "video_C",
                vec![
                    simple_tag("video"),
                    kv_tag("rating", "3"),
                    kv_tag("type", "skate"),
                ],
            ),
        ])
    }

    fn media(id: &str, tags: Vec<Tag>) -> MediaFile {
        use std::collections::HashMap as Map;

        let mut attributes = Map::new();
        for tag in &tags {
            if matches!(tag.kind, TagKind::KeyValue) {
                if let Some(value) = &tag.value {
                    attributes
                        .entry(tag.name.clone())
                        .or_insert_with(|| value.clone());
                }
            }
        }

        MediaFile {
            id: id.to_string(),
            relative_path: format!("{id}.png"),
            media_type: MediaType::Image,
            tags,
            attributes,
            filesize: 0,
            dimensions: None,
            duration_ms: None,
            thumbnail_path: Some(format!("/api/v1/media/{id}/thumbnail")),
            hash: None,
            indexed_at: Utc::now(),
        }
    }

    fn simple_tag(name: &str) -> Tag {
        Tag {
            raw_token: name.into(),
            kind: TagKind::Simple,
            name: name.to_lowercase(),
            value: None,
            normalized: name.to_lowercase(),
        }
    }

    fn kv_tag(key: &str, value: &str) -> Tag {
        Tag {
            raw_token: format!("{key}-{value}"),
            kind: TagKind::KeyValue,
            name: key.to_lowercase(),
            value: Some(value.to_lowercase()),
            normalized: format!("{}={}", key.to_lowercase(), value.to_lowercase()),
        }
    }
}
