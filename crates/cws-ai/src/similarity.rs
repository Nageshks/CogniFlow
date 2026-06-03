use std::collections::HashMap;

use cws_core::models::{HybridSearchConfig, SearchResult};

/// Combine semantic and keyword search results using weighted scoring.
///
/// Implements the 70/30 semantic/keyword weighting with score normalization:
/// 1. Normalize scores within each result set to [0, 1]
/// 2. Apply weights (default: 0.7 semantic, 0.3 keyword)
/// 3. For results appearing in both sets, combine weighted scores
/// 4. Sort by final combined score
pub fn combine_results(
    semantic_results: Vec<SearchResult>,
    keyword_results: Vec<SearchResult>,
    config: &HybridSearchConfig,
) -> Vec<SearchResult> {
    let semantic_normalized = normalize_scores(semantic_results);
    let keyword_normalized = normalize_scores(keyword_results);

    let mut combined: HashMap<String, SearchResult> = HashMap::new();

    // Add semantic results with weight
    for mut result in semantic_normalized {
        result.score *= config.semantic_weight;
        combined.insert(result.id.clone(), result);
    }

    // Add keyword results with weight, combining if already present
    for mut result in keyword_normalized {
        result.score *= config.keyword_weight;

        combined
            .entry(result.id.clone())
            .and_modify(|existing| {
                existing.score += result.score;
                // Keep the better snippet
                if existing.snippet.is_empty() && !result.snippet.is_empty() {
                    existing.snippet = result.snippet.clone();
                }
            })
            .or_insert(result);
    }

    let mut results: Vec<SearchResult> = combined.into_values().collect();
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results
}

/// Normalize scores in a result set to the [0, 1] range.
fn normalize_scores(results: Vec<SearchResult>) -> Vec<SearchResult> {
    if results.is_empty() {
        return results;
    }

    let max_score = results
        .iter()
        .map(|r| r.score)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_score = results
        .iter()
        .map(|r| r.score)
        .fold(f32::INFINITY, f32::min);

    let range = max_score - min_score;

    results
        .into_iter()
        .map(|mut r| {
            r.score = if range > 0.0 {
                (r.score - min_score) / range
            } else {
                1.0
            };
            r
        })
        .collect()
}
