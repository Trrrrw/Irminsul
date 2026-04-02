use reqwest::Client;
use serde::Deserialize;

use crate::admin::services::embedding::RuntimeEmbeddingConfig;

const CHUNK_SIZE: usize = 1200;
const CHUNK_OVERLAP: usize = 200;

/// 将结构化文本字段组合成用于检索和向量化的标准文档。
pub fn build_document(
    name: &str,
    subtitle: Option<&str>,
    author: Option<&str>,
    summary: Option<&str>,
    body: Option<&str>,
) -> (String, String, Vec<String>) {
    let mut header_parts = vec![format!("名称：{name}")];
    if let Some(subtitle) = subtitle.filter(|value| !value.trim().is_empty()) {
        header_parts.push(format!("副标题：{}", subtitle.trim()));
    }
    if let Some(author) = author.filter(|value| !value.trim().is_empty()) {
        header_parts.push(format!("作者：{}", author.trim()));
    }
    if let Some(summary) = summary.filter(|value| !value.trim().is_empty()) {
        header_parts.push(format!("摘要：{}", summary.trim()));
    }
    let header = header_parts.join("\n");
    let canonical_text = if let Some(body) = body.filter(|value| !value.trim().is_empty()) {
        format!("{header}\n\n正文：\n{}", body.trim())
    } else {
        header.clone()
    };

    let mut chunks = vec![header];
    if let Some(body) = body.filter(|value| !value.trim().is_empty()) {
        chunks.extend(split_text(body.trim(), CHUNK_SIZE, CHUNK_OVERLAP));
    }
    (name.to_string(), canonical_text, chunks)
}

/// 调用外部 embedding API，并返回可直接交给 `vector32()` 的 JSON 数组字符串。
pub async fn request_embedding(
    config: &RuntimeEmbeddingConfig,
    input: &str,
) -> Result<String, String> {
    let client = Client::new();
    let url = format!(
        "{}{}",
        config.base_url.trim_end_matches('/'),
        normalize_path(&config.embeddings_path)
    );
    let response = client
        .post(url)
        .bearer_auth(&config.api_key)
        .json(&serde_json::json!({
            "model": config.model,
            "input": input,
        }))
        .send()
        .await
        .map_err(|error| format!("请求 embedding API 失败: {error}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("embedding API 返回失败状态 {status}: {body}"));
    }

    let payload: EmbeddingResponse = response
        .json()
        .await
        .map_err(|error| format!("解析 embedding API 响应失败: {error}"))?;
    let embedding = payload
        .data
        .into_iter()
        .next()
        .ok_or_else(|| "embedding API 未返回向量数据".to_string())?
        .embedding;
    serde_json::to_string(&embedding).map_err(|error| format!("序列化 embedding 失败: {error}"))
}

fn split_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        let end = (start + chunk_size).min(chars.len());
        let chunk = chars[start..end].iter().collect::<String>();
        if !chunk.trim().is_empty() {
            chunks.push(chunk);
        }
        if end == chars.len() {
            break;
        }
        start = end.saturating_sub(overlap);
    }
    chunks
}

fn normalize_path(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}
