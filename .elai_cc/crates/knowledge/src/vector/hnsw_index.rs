use std::path::Path;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum IndexError {
    ConstructFailed(String),
    AddFailed(String),
    SearchFailed(String),
    SaveFailed(String),
    LoadFailed(String),
}

impl std::fmt::Display for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConstructFailed(e) => write!(f, "index construction failed: {e}"),
            Self::AddFailed(e) => write!(f, "add failed: {e}"),
            Self::SearchFailed(e) => write!(f, "search failed: {e}"),
            Self::SaveFailed(e) => write!(f, "save failed: {e}"),
            Self::LoadFailed(e) => write!(f, "load failed: {e}"),
        }
    }
}

impl std::error::Error for IndexError {}

// ---------------------------------------------------------------------------
// VectorIndex trait (pure Rust, no usearch dependency)
// ---------------------------------------------------------------------------

pub trait VectorIndex: Sized {
    fn add(&mut self, key: u64, vec: &[f32]) -> Result<(), IndexError>;
    fn search(&self, query: &[f32], k: usize) -> Result<Vec<(u64, f32)>, IndexError>;
    fn filtered_search(
        &self,
        query: &[f32],
        k: usize,
        predicate: &dyn Fn(u64) -> bool,
    ) -> Result<Vec<(u64, f32)>, IndexError>;
    fn save(&self, path: &Path) -> Result<(), IndexError>;
    fn load(path: &Path, dim: usize) -> Result<Self, IndexError>;
}

// ---------------------------------------------------------------------------
// UsearchIndex (only compiled when feature "usearch-hnsw" is enabled)
// ---------------------------------------------------------------------------

#[cfg(feature = "usearch-hnsw")]
pub struct UsearchIndex {
    index: usearch::Index,
    dim: usize,
}

#[cfg(feature = "usearch-hnsw")]
impl UsearchIndex {
    pub fn new(dim: usize) -> Result<Self, IndexError> {
        let opts = usearch::IndexOptions {
            dimensions: dim,
            metric: usearch::MetricKind::Cos,
            quantization: usearch::ScalarKind::F32,
            ..Default::default()
        };
        let index = usearch::Index::new(&opts)
            .map_err(|e| IndexError::ConstructFailed(format!("{e}")))?;
        Ok(Self { index, dim })
    }

    pub fn dim(&self) -> usize {
        self.dim
    }
}

#[cfg(feature = "usearch-hnsw")]
impl VectorIndex for UsearchIndex {
    fn add(&mut self, key: u64, vec: &[f32]) -> Result<(), IndexError> {
        self.index
            .add(key, vec)
            .map_err(|e| IndexError::AddFailed(format!("{e}")))
    }

    fn search(&self, query: &[f32], k: usize) -> Result<Vec<(u64, f32)>, IndexError> {
        let results = self
            .index
            .search(query, k)
            .map_err(|e| IndexError::SearchFailed(format!("{e}")))?;
        Ok(results
            .keys
            .into_iter()
            .zip(results.distances.into_iter())
            .collect())
    }

    fn filtered_search(
        &self,
        query: &[f32],
        k: usize,
        predicate: &dyn Fn(u64) -> bool,
    ) -> Result<Vec<(u64, f32)>, IndexError> {
        let results = self
            .index
            .filtered_search(query, k, predicate)
            .map_err(|e| IndexError::SearchFailed(format!("{e}")))?;
        Ok(results
            .keys
            .into_iter()
            .zip(results.distances.into_iter())
            .collect())
    }

    fn save(&self, path: &Path) -> Result<(), IndexError> {
        let path_str = path
            .to_str()
            .ok_or_else(|| IndexError::SaveFailed("invalid path encoding".into()))?;
        self.index
            .save(path_str)
            .map_err(|e| IndexError::SaveFailed(format!("{e}")))
    }

    fn load(path: &Path, dim: usize) -> Result<Self, IndexError> {
        let path_str = path
            .to_str()
            .ok_or_else(|| IndexError::LoadFailed("invalid path encoding".into()))?;
        let opts = usearch::IndexOptions {
            dimensions: dim,
            metric: usearch::MetricKind::Cos,
            quantization: usearch::ScalarKind::F32,
            ..Default::default()
        };
        let index = usearch::Index::new(&opts)
            .map_err(|e| IndexError::LoadFailed(format!("create: {e}")))?;
        index
            .load(path_str)
            .map_err(|e| IndexError::LoadFailed(format!("load: {e}")))?;
        Ok(Self { index, dim })
    }
}

// ---------------------------------------------------------------------------
// Tests (trait-level tests that don't require usearch)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "usearch-hnsw")]
    #[test]
    fn usearch_load_missing_returns_err() {
        let result = UsearchIndex::load(Path::new("/nonexistent/index.usearch"), 768);
        assert!(result.is_err());
        match result.unwrap_err() {
            IndexError::LoadFailed(_) => {}
            other => panic!("expected LoadFailed, got {other:?}"),
        }
    }

    #[test]
    fn index_error_display() {
        let err = IndexError::ConstructFailed("test".into());
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn vector_index_trait_is_object_safe_check() {
        // VectorIndex requires Sized, so it's NOT object-safe (by design).
        // This test verifies the trait compiles and can be used as a bound.
        fn _accepts_index<T: VectorIndex>(_idx: &T) {}
    }

    // --- Step 3 tests (feature-gated) ---

    #[cfg(feature = "usearch-hnsw")]
    #[test]
    fn test_hnsw_index_persistence_roundtrip() {
        let dim = 8; // small dim for test speed
        let mut idx = UsearchIndex::new(dim).unwrap();

        // Add 10 vectors
        for key in 0u64..10 {
            let vec: Vec<f32> = (0..dim).map(|d| (key as f32) * 0.1 + (d as f32) * 0.01).collect();
            idx.add(key, &vec).unwrap();
        }

        // Search before save
        let query: Vec<f32> = (0..dim).map(|d| 0.5 + (d as f32) * 0.01).collect();
        let before = idx.search(&query, 3).unwrap();
        assert_eq!(before.len(), 3);

        // Save to tempfile, load into new index, search again
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();
        // usearch needs the file to not exist for save
        drop(tmp);
        idx.save(&path).unwrap();

        let loaded = UsearchIndex::load(&path, dim).unwrap();
        let after = loaded.search(&query, 3).unwrap();
        assert_eq!(after.len(), 3);

        // Same keys returned (order may vary by float rounding, check set equality)
        let before_keys: std::collections::HashSet<u64> = before.iter().map(|(k, _)| *k).collect();
        let after_keys: std::collections::HashSet<u64> = after.iter().map(|(k, _)| *k).collect();
        assert_eq!(before_keys, after_keys, "same keys after persistence roundtrip");

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[cfg(feature = "usearch-hnsw")]
    #[test]
    fn test_hnsw_filtered_search_project_isolation() {
        let dim = 8;
        let mut idx = UsearchIndex::new(dim).unwrap();

        // Keys 0-4: project A, keys 5-9: project B
        for key in 0u64..10 {
            let vec: Vec<f32> = (0..dim).map(|d| (key as f32) * 0.1 + (d as f32) * 0.01).collect();
            idx.add(key, &vec).unwrap();
        }

        let query: Vec<f32> = (0..dim).map(|d| 0.3 + (d as f32) * 0.01).collect();

        // Filtered search: only project A keys (0-4)
        let results = idx
            .filtered_search(&query, 5, &|key| key < 5)
            .unwrap();

        for (key, _score) in &results {
            assert!(*key < 5, "filtered_search should only return project A keys, got {key}");
        }
        assert!(!results.is_empty(), "filtered_search should return at least one result");
    }
}
