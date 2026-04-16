use std::collections::{HashMap, HashSet};
use rust_stemmers::{Algorithm, Stemmer};

pub struct SearchEngine {
    stemmer: Stemmer,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            stemmer: Stemmer::create(Algorithm::English),
        }
    }

    pub fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .map(|s| self.stemmer.stem(s).to_string())
            .filter(|s| s.len() > 1)
            .collect()
    }

    pub fn compute_tfidf(&self, query: &str, documents: &[String]) -> Vec<(usize, f32)> {
        if documents.is_empty() {
            return Vec::new();
        }

        let query_tokens = self.tokenize(query);
        let doc_tokens: Vec<Vec<String>> = documents.iter().map(|d| self.tokenize(d)).collect();
        
        // IDF
        let mut df: HashMap<String, usize> = HashMap::new();
        for tokens in &doc_tokens {
            let unique: HashSet<&String> = tokens.iter().collect();
            for token in unique {
                *df.entry(token.clone()).or_insert(0) += 1;
            }
        }

        let num_docs = documents.len() as f32;
        let idf: HashMap<String, f32> = df.iter()
            .map(|(token, count)| (token.clone(), (num_docs / (*count as f32)).ln()))
            .collect();

        // Scores
        let mut scores: Vec<(usize, f32)> = Vec::new();
        for (idx, tokens) in doc_tokens.iter().enumerate() {
            let mut tf: HashMap<String, usize> = HashMap::new();
            for token in tokens {
                *tf.entry(token.clone()).or_insert(0) += 1;
            }

            let mut score = 0.0;
            for q_token in &query_tokens {
                if let Some(token_tf) = tf.get(q_token) {
                    let token_idf = idf.get(q_token).unwrap_or(&0.0);
                    score += (*token_tf as f32 / tokens.len() as f32) * token_idf;
                }
            }
            if score > 0.0 {
                scores.push((idx, score));
            }
        }

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores
    }
}
