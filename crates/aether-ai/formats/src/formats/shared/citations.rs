//! Provider-neutral source citations.
//!
//! Some providers ground an answer server-side (Gemini's native `googleSearch`
//! is the motivating case): the search leaves no client-visible tool call, and
//! the evidence arrives only as provider-specific metadata alongside the text.
//! Dropping it leaves callers with prose that names its sources but nothing
//! they can render, link, or verify.
//!
//! Adapters therefore normalise that metadata into the neutral citation shape
//! below, and each target renders it into its own family's standard shape.
//! Neither side has to learn the other's vocabulary.

use serde_json::{Map, Value};

/// Build one neutral citation.
///
/// `start_index` / `end_index` are character offsets into the answer text —
/// providers that report byte offsets convert before calling. Every field but
/// `url` is optional, because providers routinely ground an answer without
/// anchoring it to a span.
pub(crate) fn canonical_citation(
    url: &str,
    title: Option<&str>,
    start_index: Option<usize>,
    end_index: Option<usize>,
    cited_text: Option<&str>,
) -> Value {
    let mut citation = Map::new();
    citation.insert("url".to_string(), Value::String(url.to_string()));
    if let Some(title) = title {
        citation.insert("title".to_string(), Value::String(title.to_string()));
    }
    if let Some(start_index) = start_index {
        citation.insert("start_index".to_string(), Value::from(start_index as u64));
    }
    if let Some(end_index) = end_index {
        citation.insert("end_index".to_string(), Value::from(end_index as u64));
    }
    if let Some(cited_text) = cited_text {
        citation.insert(
            "cited_text".to_string(),
            Value::String(cited_text.to_string()),
        );
    }
    Value::Object(citation)
}

fn citation_string<'a>(citation: &'a Value, key: &str) -> Option<&'a str> {
    citation
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

/// Render a neutral citation as an OpenAI `url_citation` annotation, the shape
/// both `chat.completions` and `responses` attach to assistant text.
pub(crate) fn canonical_citation_to_openai_annotation(citation: &Value) -> Option<Value> {
    let url = citation_string(citation, "url")?;
    let mut annotation = Map::new();
    annotation.insert(
        "type".to_string(),
        Value::String("url_citation".to_string()),
    );
    annotation.insert("url".to_string(), Value::String(url.to_string()));
    if let Some(title) = citation_string(citation, "title") {
        annotation.insert("title".to_string(), Value::String(title.to_string()));
    }
    for key in ["start_index", "end_index"] {
        if let Some(index) = citation.get(key).and_then(Value::as_u64) {
            annotation.insert(key.to_string(), Value::from(index));
        }
    }
    Some(Value::Object(annotation))
}

/// Render a neutral citation as a Claude `web_search_result_location`, the
/// shape Claude puts in a text block's `citations`.
pub(crate) fn canonical_citation_to_claude_citation(citation: &Value) -> Option<Value> {
    let url = citation_string(citation, "url")?;
    let mut out = Map::new();
    out.insert(
        "type".to_string(),
        Value::String("web_search_result_location".to_string()),
    );
    out.insert("url".to_string(), Value::String(url.to_string()));
    if let Some(title) = citation_string(citation, "title") {
        out.insert("title".to_string(), Value::String(title.to_string()));
    }
    if let Some(cited_text) = citation_string(citation, "cited_text") {
        out.insert(
            "cited_text".to_string(),
            Value::String(cited_text.to_string()),
        );
    }
    Some(Value::Object(out))
}

/// Render every citation that carries a usable URL.
pub(crate) fn canonical_citations_to_openai_annotations(citations: &[Value]) -> Vec<Value> {
    citations
        .iter()
        .filter_map(canonical_citation_to_openai_annotation)
        .collect()
}

/// Render every citation that carries a usable URL.
pub(crate) fn canonical_citations_to_claude_citations(citations: &[Value]) -> Vec<Value> {
    citations
        .iter()
        .filter_map(canonical_citation_to_claude_citation)
        .collect()
}
