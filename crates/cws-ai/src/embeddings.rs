use rand::Rng;
use tracing::info;

use cws_core::error::CwsResult;
use cws_core::traits::EmbeddingEngine;

const EMBEDDING_DIM: usize = 384;

/// Stub embedding engine that generates random vectors.
/// TODO: Replace with ONNX Runtime integration for real embeddings.
///
/// When the real model is integrated:
/// - Download all-MiniLM-L6-v2 ONNX model
/// - Load using ort (ONNX Runtime) crate
/// - Tokenize input with tokenizers crate
/// - Run inference to get real 384-dim embeddings
pub struct StubEmbeddingEngine {
    _model_available: bool,
}

impl StubEmbeddingEngine {
    pub fn new() -> Self {
        info!("Stub embedding engine initialized (random vectors)");
        Self {
            _model_available: false,
        }
    }

    /// Check if the real ONNX model is available.
    pub fn is_model_available(&self) -> bool {
        self._model_available
    }

    /// Get the URL where the model can be downloaded.
    pub fn model_download_url(&self) -> &'static str {
        "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/tree/main/onnx"
    }
}

impl Default for StubEmbeddingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbeddingEngine for StubEmbeddingEngine {
    fn embed(&self, _text: &str) -> CwsResult<Vec<f32>> {
        // TODO: Replace with actual ONNX inference
        let mut rng = rand::thread_rng();
        let embedding: Vec<f32> = (0..EMBEDDING_DIM).map(|_| rng.gen_range(-1.0..1.0)).collect();

        // Normalize to unit vector
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        let normalized: Vec<f32> = if norm > 0.0 {
            embedding.iter().map(|x| x / norm).collect()
        } else {
            embedding
        };

        Ok(normalized)
    }

    fn embed_batch(&self, texts: &[&str]) -> CwsResult<Vec<Vec<f32>>> {
        // TODO: Replace with batched ONNX inference for efficiency
        texts.iter().map(|text| self.embed(text)).collect()
    }

    fn dimensions(&self) -> usize {
        EMBEDDING_DIM
    }
}
