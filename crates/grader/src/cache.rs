//! Grade caching using SQLite and content hashing
//!
//! Uses SHA-256 to hash artifact content and stores grades in SQLite
//! to avoid redundant API calls for identical content.

use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::error::GraderError;
use crate::types::{CategoryScore, GradeResult};

/// Cache for storing and retrieving grades
pub struct GradeCache {
    conn: Connection,
}

impl GradeCache {
    /// Create a new grade cache with the given database path
    pub fn new(db_path: &Path) -> Result<Self, GraderError> {
        let conn = Connection::open(db_path)?;
        let cache = Self { conn };
        cache.init_schema()?;
        Ok(cache)
    }

    /// Create an in-memory cache (for testing)
    pub fn in_memory() -> Result<Self, GraderError> {
        let conn = Connection::open_in_memory()?;
        let cache = Self { conn };
        cache.init_schema()?;
        Ok(cache)
    }

    /// Initialize the database schema
    fn init_schema(&self) -> Result<(), GraderError> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS grade_cache (
                content_hash TEXT PRIMARY KEY,
                artifact_type TEXT NOT NULL,
                grade INTEGER NOT NULL,
                overall_feedback TEXT NOT NULL,
                category_scores TEXT NOT NULL,
                cached_at TEXT NOT NULL,
                hit_count INTEGER DEFAULT 0
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_grade_cache_type ON grade_cache(artifact_type)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_grade_cache_date ON grade_cache(cached_at)",
            [],
        )?;

        Ok(())
    }

    /// Get a cached grade for the given content
    pub fn get(&self, content: &str, artifact_type: &str) -> Result<Option<GradeResult>, GraderError> {
        let hash = Self::hash_content(content);

        let mut stmt = self.conn.prepare(
            "SELECT grade, overall_feedback, category_scores, cached_at
             FROM grade_cache
             WHERE content_hash = ?1 AND artifact_type = ?2",
        )?;

        let result = stmt.query_row(params![hash, artifact_type], |row| {
            let grade: u32 = row.get(0)?;
            let overall_feedback: String = row.get(1)?;
            let category_scores_json: String = row.get(2)?;
            let _cached_at: String = row.get(3)?;

            let category_scores: Vec<CategoryScore> =
                serde_json::from_str(&category_scores_json).unwrap_or_default();

            Ok(GradeResult {
                score: grade,
                max_score: 100,
                overall_feedback,
                category_scores,
                from_cache: true,
                latency_ms: 0,
            })
        });

        match result {
            Ok(grade) => {
                // Increment hit count
                let _ = self.conn.execute(
                    "UPDATE grade_cache SET hit_count = hit_count + 1
                     WHERE content_hash = ?1",
                    params![hash],
                );
                Ok(Some(grade))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Store a grade in the cache
    pub fn set(
        &self,
        content: &str,
        artifact_type: &str,
        result: &GradeResult,
    ) -> Result<(), GraderError> {
        let hash = Self::hash_content(content);
        let now = chrono::Utc::now().to_rfc3339();
        let scores_json = serde_json::to_string(&result.category_scores)
            .map_err(|e| GraderError::CacheError(e.to_string()))?;

        self.conn.execute(
            "INSERT INTO grade_cache (content_hash, artifact_type, grade, overall_feedback, category_scores, cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(content_hash) DO UPDATE SET
                grade = excluded.grade,
                overall_feedback = excluded.overall_feedback,
                category_scores = excluded.category_scores,
                cached_at = excluded.cached_at",
            params![
                hash,
                artifact_type,
                result.score,
                result.overall_feedback,
                scores_json,
                now
            ],
        )?;

        Ok(())
    }

    /// Hash content with normalization
    pub fn hash_content(content: &str) -> String {
        let mut hasher = Sha256::new();

        // Normalize content before hashing to improve cache hits
        let normalized = content
            .lines()
            .map(|line| line.trim_end()) // Remove trailing whitespace
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();

        hasher.update(normalized.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<CacheStats, GraderError> {
        let (total_entries, total_hits): (i64, Option<i64>) = self.conn.query_row(
            "SELECT COUNT(*), SUM(hit_count) FROM grade_cache",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        Ok(CacheStats {
            total_entries: total_entries as usize,
            total_hits: total_hits.unwrap_or(0) as usize,
        })
    }

    /// Clear old entries from the cache
    pub fn cleanup_old_entries(&self, days: u32) -> Result<usize, GraderError> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
        let cutoff_str = cutoff.to_rfc3339();

        let deleted = self.conn.execute(
            "DELETE FROM grade_cache WHERE cached_at < ?1",
            params![cutoff_str],
        )?;

        Ok(deleted)
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_hits: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_new_in_memory() {
        let cache = GradeCache::in_memory().unwrap();
        let stats = cache.stats().unwrap();
        assert_eq!(stats.total_entries, 0);
    }

    #[test]
    fn test_cache_set_and_get() {
        let cache = GradeCache::in_memory().unwrap();

        let result = GradeResult::new(85, "Good work!".to_string(), vec![], 500);

        // Store
        cache.set("# Test Content", "DESIGN", &result).unwrap();

        // Retrieve
        let cached = cache.get("# Test Content", "DESIGN").unwrap();
        assert!(cached.is_some());
        let cached = cached.unwrap();
        assert_eq!(cached.score, 85);
        assert_eq!(cached.overall_feedback, "Good work!");
        assert!(cached.from_cache);
    }

    #[test]
    fn test_cache_miss() {
        let cache = GradeCache::in_memory().unwrap();

        let cached = cache.get("nonexistent", "DESIGN").unwrap();
        assert!(cached.is_none());
    }

    #[test]
    fn test_hash_normalization() {
        // Same content with different whitespace
        let content1 = "# Test  \n\nSome content";
        let content2 = "# Test\n\nSome content";

        let hash1 = GradeCache::hash_content(content1);
        let hash2 = GradeCache::hash_content(content2);

        assert_eq!(hash1, hash2, "Hashes should match after normalization");
    }

    #[test]
    fn test_hash_different_content() {
        let content1 = "# Test A";
        let content2 = "# Test B";

        let hash1 = GradeCache::hash_content(content1);
        let hash2 = GradeCache::hash_content(content2);

        assert_ne!(hash1, hash2, "Different content should have different hashes");
    }

    #[test]
    fn test_cache_hit_counter() {
        let cache = GradeCache::in_memory().unwrap();

        let result = GradeResult::new(85, "Good!".to_string(), vec![], 0);
        cache.set("content", "DESIGN", &result).unwrap();

        // Get multiple times
        cache.get("content", "DESIGN").unwrap();
        cache.get("content", "DESIGN").unwrap();
        cache.get("content", "DESIGN").unwrap();

        let stats = cache.stats().unwrap();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.total_hits, 3);
    }

    #[test]
    fn test_cache_with_category_scores() {
        let cache = GradeCache::in_memory().unwrap();

        let scores = vec![
            CategoryScore::new("Architecture".to_string(), 20, 25, "Good structure".to_string()),
            CategoryScore::new("Documentation".to_string(), 18, 25, "Clear docs".to_string()),
        ];

        let result = GradeResult::new(85, "Overall good".to_string(), scores, 500);
        cache.set("content", "DESIGN", &result).unwrap();

        let cached = cache.get("content", "DESIGN").unwrap().unwrap();
        assert_eq!(cached.category_scores.len(), 2);
        assert_eq!(cached.category_scores[0].category, "Architecture");
        assert_eq!(cached.category_scores[0].score, 20);
    }

    #[test]
    fn test_cache_update() {
        let cache = GradeCache::in_memory().unwrap();

        // First grade
        let result1 = GradeResult::new(75, "OK".to_string(), vec![], 0);
        cache.set("content", "DESIGN", &result1).unwrap();

        // Update with new grade
        let result2 = GradeResult::new(85, "Better!".to_string(), vec![], 0);
        cache.set("content", "DESIGN", &result2).unwrap();

        // Should get updated value
        let cached = cache.get("content", "DESIGN").unwrap().unwrap();
        assert_eq!(cached.score, 85);
        assert_eq!(cached.overall_feedback, "Better!");
    }

    #[test]
    fn test_different_artifact_types() {
        let cache = GradeCache::in_memory().unwrap();

        let result = GradeResult::new(85, "Good".to_string(), vec![], 0);
        cache.set("content", "DESIGN", &result).unwrap();

        // Same content, different type
        let cached = cache.get("content", "README").unwrap();
        assert!(cached.is_none());

        // Same content, same type
        let cached = cache.get("content", "DESIGN").unwrap();
        assert!(cached.is_some());
    }
}
