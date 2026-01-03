//! Container pool for pre-warming
//!
//! Keeps a pool of warm containers ready to reduce cold-start latency.

use std::collections::VecDeque;
use tokio::sync::Mutex;

use crate::types::DockerConfig;

/// A pool of pre-warmed containers
/// 
/// Note: This is a simplified implementation. The actual container pre-warming
/// would require more sophisticated lifecycle management. For MVP, we create
/// containers on-demand and this pool serves as a placeholder for the pattern.
pub struct ContainerPool {
    /// Queue of available container IDs
    idle: Mutex<VecDeque<String>>,
    /// Configuration for creating containers (reserved for future use)
    #[allow(dead_code)]
    config: DockerConfig,
    /// Maximum pool size
    max_size: usize,
}

impl ContainerPool {
    /// Create a new container pool
    pub fn new(config: DockerConfig) -> Self {
        let max_size = config.pre_warm_pool_size;
        Self {
            idle: Mutex::new(VecDeque::new()),
            config,
            max_size,
        }
    }

    /// Get a container from the pool, or None if empty
    pub async fn get(&self) -> Option<String> {
        let mut idle = self.idle.lock().await;
        idle.pop_front()
    }

    /// Return a container to the pool
    pub async fn return_container(&self, container_id: String) {
        let mut idle = self.idle.lock().await;
        
        // Only return if pool is not full
        if idle.len() < self.max_size {
            idle.push_back(container_id);
        }
        // If pool is full, the container should be destroyed by the caller
    }

    /// Check how many containers are available
    pub async fn available(&self) -> usize {
        let idle = self.idle.lock().await;
        idle.len()
    }

    /// Get the maximum pool size
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Check if the pool is full
    pub async fn is_full(&self) -> bool {
        let idle = self.idle.lock().await;
        idle.len() >= self.max_size
    }

    /// Clear the pool (returns all container IDs for cleanup)
    pub async fn drain(&self) -> Vec<String> {
        let mut idle = self.idle.lock().await;
        idle.drain(..).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_new() {
        let config = DockerConfig::default();
        let pool = ContainerPool::new(config);
        
        assert_eq!(pool.available().await, 0);
        assert_eq!(pool.max_size(), 2);
    }

    #[tokio::test]
    async fn test_pool_get_empty() {
        let config = DockerConfig::default();
        let pool = ContainerPool::new(config);
        
        assert!(pool.get().await.is_none());
    }

    #[tokio::test]
    async fn test_pool_return_and_get() {
        let config = DockerConfig::default();
        let pool = ContainerPool::new(config);
        
        // Return a container
        pool.return_container("container-1".to_string()).await;
        assert_eq!(pool.available().await, 1);
        
        // Get it back
        let container = pool.get().await;
        assert_eq!(container, Some("container-1".to_string()));
        assert_eq!(pool.available().await, 0);
    }

    #[tokio::test]
    async fn test_pool_respects_max_size() {
        let mut config = DockerConfig::default();
        config.pre_warm_pool_size = 2;
        let pool = ContainerPool::new(config);
        
        // Return 3 containers (max is 2)
        pool.return_container("container-1".to_string()).await;
        pool.return_container("container-2".to_string()).await;
        pool.return_container("container-3".to_string()).await;
        
        // Only 2 should be in pool
        assert_eq!(pool.available().await, 2);
        assert!(pool.is_full().await);
    }

    #[tokio::test]
    async fn test_pool_fifo_order() {
        let config = DockerConfig::default();
        let pool = ContainerPool::new(config);
        
        pool.return_container("first".to_string()).await;
        pool.return_container("second".to_string()).await;
        
        // Should get in FIFO order
        assert_eq!(pool.get().await, Some("first".to_string()));
        assert_eq!(pool.get().await, Some("second".to_string()));
    }

    #[tokio::test]
    async fn test_pool_drain() {
        let config = DockerConfig::default();
        let pool = ContainerPool::new(config);
        
        pool.return_container("c1".to_string()).await;
        pool.return_container("c2".to_string()).await;
        
        let drained = pool.drain().await;
        assert_eq!(drained.len(), 2);
        assert_eq!(pool.available().await, 0);
    }
}
