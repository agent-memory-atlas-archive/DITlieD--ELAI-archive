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
// BruteForceIndex — pure-Rust cosine-similarity search, no native deps.
// Suitable for project-scale corpora (hundreds to low thousands of vectors).
// ---------------------------------------------------------------------------

pub struct BruteForceIndex {
    dim: usize,
    keys: Vec<u64>,
    vecs: Vec<Vec<f32>>,
}

impl BruteForceIndex {
    pub fn new(dim: usize) -> Self {
        Self { dim, keys: Vec::new(), vecs: Vec::new() }
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Remove a key and its vector. Returns true if found.
    pub fn remove(&mut self, key: u64) -> bool {
        if let Some(pos) = self.keys.iter().position(|&k| k == key) {
            self.keys.swap_remove(pos);
            self.vecs.swap_remove(pos);
            true
        } else {
            false
        }
    }

    fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
        let mut dot = 0.0f64;
        let mut norm_a = 0.0f64;
        let mut norm_b = 0.0f64;
        for (x, y) in a.iter().zip(b.iter()) {
            let xd = *x as f64;
            let yd = *y as f64;
            dot += xd * yd;
            norm_a += xd * xd;
            norm_b += yd * yd;
        }
        let denom = norm_a.sqrt() * norm_b.sqrt();
        if denom == 0.0 { 0.0 } else { (dot / denom) as f32 }
    }

    fn search_with_filter(
        &self,
        query: &[f32],
        k: usize,
        predicate: Option<&dyn Fn(u64) -> bool>,
    ) -> Result<Vec<(u64, f32)>, IndexError> {
        if query.len() != self.dim {
            return Err(IndexError::SearchFailed(format!(
                "query dim {} != index dim {}",
                query.len(),
                self.dim
            )));
        }
        let mut scored: Vec<(u64, f32)> = self
            .keys
            .iter()
            .zip(self.vecs.iter())
            .filter(|(&key, _)| predicate.map_or(true, |p| p(key)))
            .map(|(&key, vec)| (key, Self::cosine_sim(query, vec)))
            .collect();
        // Sort descending by similarity.
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        Ok(scored)
    }
}

impl VectorIndex for BruteForceIndex {
    fn add(&mut self, key: u64, vec: &[f32]) -> Result<(), IndexError> {
        if vec.len() != self.dim {
            return Err(IndexError::AddFailed(format!(
                "vec dim {} != index dim {}",
                vec.len(),
                self.dim
            )));
        }
        // Upsert: if key exists, replace its vector.
        if let Some(pos) = self.keys.iter().position(|&k| k == key) {
            self.vecs[pos] = vec.to_vec();
        } else {
            self.keys.push(key);
            self.vecs.push(vec.to_vec());
        }
        Ok(())
    }

    fn search(&self, query: &[f32], k: usize) -> Result<Vec<(u64, f32)>, IndexError> {
        self.search_with_filter(query, k, None)
    }

    fn filtered_search(
        &self,
        query: &[f32],
        k: usize,
        predicate: &dyn Fn(u64) -> bool,
    ) -> Result<Vec<(u64, f32)>, IndexError> {
        self.search_with_filter(query, k, Some(predicate))
    }

    fn save(&self, path: &Path) -> Result<(), IndexError> {
        let data = BruteForceData {
            dim: self.dim,
            keys: self.keys.clone(),
            vecs: self.vecs.clone(),
        };
        let bytes = serde_json::to_vec(&data)
            .map_err(|e| IndexError::SaveFailed(format!("serialize: {e}")))?;
        std::fs::write(path, bytes)
            .map_err(|e| IndexError::SaveFailed(format!("write: {e}")))?;
        Ok(())
    }

    fn load(path: &Path, dim: usize) -> Result<Self, IndexError> {
        let bytes = std::fs::read(path)
            .map_err(|e| IndexError::LoadFailed(format!("read: {e}")))?;
        let data: BruteForceData = serde_json::from_slice(&bytes)
            .map_err(|e| IndexError::LoadFailed(format!("deserialize: {e}")))?;
        if data.dim != dim {
            return Err(IndexError::LoadFailed(format!(
                "dim mismatch: file has {}, expected {dim}",
                data.dim
            )));
        }
        Ok(Self {
            dim: data.dim,
            keys: data.keys,
            vecs: data.vecs,
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct BruteForceData {
    dim: usize,
    keys: Vec<u64>,
    vecs: Vec<Vec<f32>>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brute_force_add_and_search() {
        let dim = 4;
        let mut idx = BruteForceIndex::new(dim);
        idx.add(1, &[1.0, 0.0, 0.0, 0.0]).unwrap();
        idx.add(2, &[0.0, 1.0, 0.0, 0.0]).unwrap();
        idx.add(3, &[0.9, 0.1, 0.0, 0.0]).unwrap();

        let results = idx.search(&[1.0, 0.0, 0.0, 0.0], 2).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 1); // exact match is most similar
        assert_eq!(results[1].0, 3); // nearly aligned
    }

    #[test]
    fn brute_force_filtered_search() {
        let dim = 4;
        let mut idx = BruteForceIndex::new(dim);
        for key in 0u64..10 {
            let v: Vec<f32> = (0..dim).map(|d| (key as f32) * 0.1 + (d as f32) * 0.01).collect();
            idx.add(key, &v).unwrap();
        }
        let query: Vec<f32> = (0..dim).map(|d| 0.5 + (d as f32) * 0.01).collect();
        let results = idx.filtered_search(&query, 5, &|key| key < 5).unwrap();
        for (key, _) in &results {
            assert!(*key < 5);
        }
    }

    #[test]
    fn brute_force_upsert() {
        let mut idx = BruteForceIndex::new(2);
        idx.add(1, &[1.0, 0.0]).unwrap();
        idx.add(1, &[0.0, 1.0]).unwrap(); // upsert
        assert_eq!(idx.len(), 1);
        let results = idx.search(&[0.0, 1.0], 1).unwrap();
        assert_eq!(results[0].0, 1);
        assert!((results[0].1 - 1.0).abs() < 0.01); // exact match after upsert
    }

    #[test]
    fn brute_force_remove() {
        let mut idx = BruteForceIndex::new(2);
        idx.add(1, &[1.0, 0.0]).unwrap();
        idx.add(2, &[0.0, 1.0]).unwrap();
        assert!(idx.remove(1));
        assert_eq!(idx.len(), 1);
        assert!(!idx.remove(99));
    }

    #[test]
    fn brute_force_persistence_roundtrip() {
        let dim = 4;
        let mut idx = BruteForceIndex::new(dim);
        for key in 0u64..5 {
            let v: Vec<f32> = (0..dim).map(|d| (key as f32) * 0.1 + (d as f32) * 0.01).collect();
            idx.add(key, &v).unwrap();
        }

        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();
        idx.save(&path).unwrap();

        let loaded = BruteForceIndex::load(&path, dim).unwrap();
        assert_eq!(loaded.len(), 5);

        let query: Vec<f32> = (0..dim).map(|d| 0.3 + (d as f32) * 0.01).collect();
        let before = idx.search(&query, 3).unwrap();
        let after = loaded.search(&query, 3).unwrap();
        let keys_before: std::collections::HashSet<u64> = before.iter().map(|(k, _)| *k).collect();
        let keys_after: std::collections::HashSet<u64> = after.iter().map(|(k, _)| *k).collect();
        assert_eq!(keys_before, keys_after);
    }

    #[test]
    fn brute_force_dim_mismatch_errors() {
        let mut idx = BruteForceIndex::new(4);
        assert!(idx.add(1, &[1.0, 0.0]).is_err()); // wrong dim
        assert!(idx.search(&[1.0, 0.0], 1).is_err()); // wrong dim
    }

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
