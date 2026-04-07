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
}
