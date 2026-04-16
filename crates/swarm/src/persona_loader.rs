use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A parsed persona from the agency-agents library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    /// Display name derived from filename (e.g. "Senior Developer")
    pub name: String,
    /// Category/domain (e.g. "engineering", "marketing", "design")
    pub category: String,
    /// Full filename (e.g. "engineering-senior-developer.md")
    pub filename: String,
    /// Absolute path to the persona file
    pub path: PathBuf,
    /// The raw markdown content of the persona file
    pub content: String,
    /// First paragraph or description extracted from the file
    pub description: String,
    /// Keywords extracted from the filename and content for matching
    pub keywords: Vec<String>,
}

/// A catalog of all available personas from the agency-agents library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaCatalog {
    pub personas: Vec<Persona>,
    /// Index by category â†’ persona names
    pub by_category: HashMap<String, Vec<String>>,
    /// Total count
    pub count: usize,
}

impl PersonaCatalog {
    /// Load all personas from the agency-agents library directory.
    pub fn load(library_path: &Path) -> Result<Self> {
        let agent_library_subdir = std::env::var("POKEDEX_AGENT_LIBRARY").unwrap_or_else(|_| "agency-agents".to_string());
        let agents_dir = library_path.join(agent_library_subdir);
        if !agents_dir.exists() {
            anyhow::bail!(
                "Agency-agents library not found at: {}",
                agents_dir.display()
            );
        }

        let mut personas = Vec::new();
        let mut by_category: HashMap<String, Vec<String>> = HashMap::new();

        for entry in WalkDir::new(&agents_dir)
            .min_depth(2) // skip root and category dirs themselves
            .max_depth(2) // only go into category/file.md
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                // Skip non-persona files
                let filename = path.file_name().unwrap().to_string_lossy().to_string();
                if filename == "README.md"
                    || filename == "CONTRIBUTING.md"
                    || filename == "LICENSE"
                {
                    continue;
                }

                if let Ok(persona) = parse_persona(path) {
                    by_category
                        .entry(persona.category.clone())
                        .or_default()
                        .push(persona.name.clone());
                    personas.push(persona);
                }
            }
        }

        let count = personas.len();
        tracing::info!("Loaded {} personas from agency-agents library", count);

        Ok(PersonaCatalog {
            personas,
            by_category,
            count,
        })
    }

    /// Find personas matching a role description by weighted scoring.
    /// Prioritizes domain-critical terms and reduces noise from common titles.
    pub fn find_matching(&self, role_description: &str) -> Vec<&Persona> {
        let query_lower = role_description.to_lowercase();
        let query_words: Vec<&str> = query_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .collect();

        // Specific high-priority domain terms
        let domain_words = ["security", "pentest", "devsecops", "psychology", "ux", "behavioral", "research"];
        // Generic "noise" words that often cause false positives
        let noise_words = ["specialist", "engineer", "expert", "agent", "architect", "lead", "senior", "specialized"];

        let mut scored: Vec<(&Persona, usize)> = self
            .personas
            .iter()
            .map(|p| {
                let mut score = 0;
                let persona_name_lower = p.name.to_lowercase();
                
                // 1. Exact Name Match (very strong signal)
                if persona_name_lower == query_lower {
                    score += 100;
                }

                for &word in &query_words {
                    let mut word_score = 0;

                    // 2. Exact word in name
                    if persona_name_lower.split_whitespace().any(|w| w == word) {
                        word_score += 25;
                    } else if persona_name_lower.contains(word) {
                        word_score += 5;
                    }

                    // 3. Exact Keyword Match
                    if p.keywords.iter().any(|k| k == word) {
                        word_score += 15;
                    }

                    // 4. Exact Category Match
                    if p.category.to_lowercase() == word {
                        word_score += 10;
                    }

                    // 5. Weighting adjustments
                    if domain_words.contains(&word) {
                        // Double the score for domain-critical matches
                        word_score *= 3;
                    } else if noise_words.contains(&word) {
                        // Down-weight generic noise words
                        word_score /= 2;
                    }

                    score += word_score;
                }
                
                (p, score)
            })
            .filter(|(_, score)| *score > 0)
            .collect();

        // Sort by score descending, then by name for deterministic results
        scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.name.cmp(&b.0.name)));
        
        scored.into_iter().map(|(p, _)| p).collect()
    }

    /// Get all unique categories.
    pub fn categories(&self) -> Vec<&str> {
        self.by_category.keys().map(|s| s.as_str()).collect()
    }

    /// Get a persona by name.
    pub fn get_by_name(&self, name: &str) -> Option<&Persona> {
        self.personas.iter().find(|p| p.name == name)
    }

    /// Get all personas in a category.
    pub fn get_by_category(&self, category: &str) -> Vec<&Persona> {
        self.personas
            .iter()
            .filter(|p| p.category == category)
            .collect()
    }
}

/// Parse a single persona markdown file into a Persona struct.
fn parse_persona(path: &Path) -> Result<Persona> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read persona file: {}", path.display()))?;

    let filename = path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    // Category is the parent directory name
    let category = path
        .parent()
        .and_then(|p| p.file_name())
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Name: remove category prefix and .md extension, convert hyphens to spaces, title case
    let name = filename
        .trim_end_matches(".md")
        .strip_prefix(&format!("{}-", category))
        .unwrap_or(filename.trim_end_matches(".md"))
        .replace('-', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    format!("{}{}", first.to_uppercase(), chars.as_str())
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    // Extract first paragraph as description
    let description = content
        .lines()
        .skip_while(|line| line.starts_with('#') || line.trim().is_empty())
        .take_while(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(200)
        .collect::<String>();

    // Keywords from filename parts and category
    let mut keywords: Vec<String> = filename
        .trim_end_matches(".md")
        .split('-')
        .map(|s| s.to_lowercase())
        .collect();
    keywords.push(category.clone());
    keywords.dedup();

    Ok(Persona {
        name,
        category,
        filename,
        path: path.to_path_buf(),
        content,
        description,
        keywords,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weighted_matching_prioritizes_domain() {
        // Create dummy personas for matching test
        let p1 = Persona {
            name: "Inclusive Visuals Specialist".to_string(),
            category: "design".to_string(),
            filename: "design-inclusive-visuals-specialist.md".to_string(),
            path: PathBuf::new(),
            content: "".to_string(),
            description: "".to_string(),
            keywords: vec!["design".into(), "inclusive".into(), "visuals".into(), "specialist".into()],
        };

        let p2 = Persona {
            name: "Security Engineer".to_string(),
            category: "engineering".to_string(),
            filename: "engineering-security-engineer.md".to_string(),
            path: PathBuf::new(),
            content: "".to_string(),
            description: "".to_string(),
            keywords: vec!["engineering".into(), "security".into(), "engineer".into()],
        };

        let catalog = PersonaCatalog {
            personas: vec![p1, p2],
            by_category: HashMap::new(),
            count: 2,
        };

        // The query that previously failed
        let role = "DevSecOps Engineer (Security Pentester & Hardening Specialist)";
        let matches = catalog.find_matching(role);

        assert!(!matches.is_empty());
        // With weighted scoring, Security Engineer (p2) MUST be first because:
        // - "security" is a domain word (x3 multiplier)
        // - "specialist" and "engineer" are noise words (/2 multiplier)
        assert_eq!(matches[0].name, "Security Engineer");
    }
}
