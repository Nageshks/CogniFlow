#![allow(unused)]

use cws_core::models::IntentClassification;

/// Suggest tags based on keyword analysis of content.
/// TODO: Replace with ML-based classification using ONNX.
pub fn suggest_tags(content: &str) -> Vec<String> {
    let lower = content.to_lowercase();
    let mut tags = Vec::new();

    let keyword_tags = [
        (&["code", "programming", "function", "class", "variable", "algorithm"][..], "coding"),
        (&["write", "essay", "blog", "article", "story", "creative"][..], "writing"),
        (&["analyze", "analysis", "data", "report", "metrics", "statistics"][..], "analysis"),
        (&["plan", "strategy", "goal", "roadmap", "milestone"][..], "planning"),
        (&["debug", "fix", "error", "bug", "issue", "troubleshoot"][..], "debugging"),
        (&["test", "testing", "qa", "quality", "validation"][..], "testing"),
        (&["design", "ui", "ux", "layout", "interface", "wireframe"][..], "design"),
        (&["api", "rest", "graphql", "endpoint", "request"][..], "api"),
        (&["database", "sql", "query", "table", "schema"][..], "database"),
        (&["deploy", "deployment", "ci", "cd", "pipeline", "docker"][..], "devops"),
        (&["security", "auth", "authentication", "encryption"][..], "security"),
        (&["email", "message", "communication", "letter"][..], "communication"),
        (&["learn", "tutorial", "guide", "explain", "teach"][..], "education"),
        (&["translate", "language", "localization"][..], "translation"),
        (&["summarize", "summary", "abstract", "tldr"][..], "summarization"),
    ];

    for (keywords, tag) in &keyword_tags {
        if keywords.iter().any(|kw| lower.contains(kw)) {
            tags.push(tag.to_string());
        }
    }

    if tags.is_empty() {
        tags.push("general".to_string());
    }

    tags.truncate(5); // Max 5 tag suggestions
    tags
}

/// Suggest a category based on keyword analysis.
/// TODO: Replace with ML-based classification using ONNX.
pub fn suggest_category(content: &str) -> Option<String> {
    let lower = content.to_lowercase();

    let categories = [
        (&["code", "programming", "software", "developer", "function"][..], "Development"),
        (&["write", "content", "blog", "article", "copy"][..], "Content Creation"),
        (&["analyze", "data", "research", "report"][..], "Research & Analysis"),
        (&["plan", "project", "manage", "organize"][..], "Project Management"),
        (&["market", "seo", "campaign", "brand", "social"][..], "Marketing"),
        (&["learn", "study", "course", "education"][..], "Education"),
        (&["design", "creative", "art", "visual"][..], "Design"),
        (&["business", "strategy", "finance", "revenue"][..], "Business"),
    ];

    for (keywords, category) in &categories {
        if keywords.iter().any(|kw| lower.contains(kw)) {
            return Some(category.to_string());
        }
    }

    None
}

/// Classify the intent of content.
/// TODO: Replace with ML-based intent classification using ONNX.
pub fn classify_intent(content: &str) -> IntentClassification {
    let lower = content.to_lowercase();

    let intents = [
        (&["generate", "create", "make", "build", "write"][..], "generation", 0.8),
        (&["analyze", "evaluate", "assess", "review"][..], "analysis", 0.75),
        (&["transform", "convert", "translate", "rewrite"][..], "transformation", 0.7),
        (&["explain", "describe", "what is", "how to"][..], "explanation", 0.7),
        (&["summarize", "shorten", "condense", "tldr"][..], "summarization", 0.75),
        (&["fix", "debug", "solve", "troubleshoot"][..], "problem-solving", 0.7),
        (&["compare", "contrast", "versus", "difference"][..], "comparison", 0.7),
        (&["brainstorm", "ideate", "suggest", "ideas"][..], "brainstorming", 0.65),
    ];

    for (keywords, intent, confidence) in &intents {
        if keywords.iter().any(|kw| lower.contains(kw)) {
            return IntentClassification {
                intent: intent.to_string(),
                confidence: *confidence,
            };
        }
    }

    IntentClassification {
        intent: "general".to_string(),
        confidence: 0.5,
    }
}
