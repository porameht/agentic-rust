//! Process Types for Crew Execution
//!
//! Defines how tasks are executed within a crew:
//! - Sequential: Tasks are executed one after another
//! - Hierarchical: A manager agent delegates tasks to workers

use serde::{Deserialize, Serialize};

/// Process type for crew execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Process {
    /// Tasks are executed sequentially, one after another
    /// Output from one task can be used as context for the next
    Sequential,

    /// A manager agent coordinates and delegates tasks to worker agents
    /// The manager decides task order and can reassign based on results
    Hierarchical,

    /// Tasks are executed in parallel where dependencies allow
    /// Respects task dependencies but maximizes parallelism
    Parallel,

    /// Custom process with user-defined execution logic
    Custom,
}

impl Default for Process {
    fn default() -> Self {
        Self::Sequential
    }
}

/// Configuration for process execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    /// The process type
    pub process_type: Process,

    /// For hierarchical process: the model to use for the manager
    pub manager_model: Option<String>,

    /// For hierarchical process: allow manager to delegate
    pub allow_delegation: bool,

    /// Maximum parallel tasks (for Parallel process)
    pub max_parallel: usize,

    /// Whether to stop on first failure
    pub fail_fast: bool,

    /// Whether to retry failed tasks
    pub retry_failed: bool,

    /// Maximum retries per task
    pub max_retries: usize,

    /// Timeout for entire crew execution in seconds
    pub crew_timeout: Option<u64>,

    /// Whether to collect intermediate outputs
    pub collect_intermediate: bool,

    /// Enable verbose logging
    pub verbose: bool,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            process_type: Process::Sequential,
            manager_model: None,
            allow_delegation: true,
            max_parallel: 4,
            fail_fast: false,
            retry_failed: true,
            max_retries: 2,
            crew_timeout: None,
            collect_intermediate: true,
            verbose: false,
        }
    }
}

impl ProcessConfig {
    /// Create a sequential process config
    pub fn sequential() -> Self {
        Self {
            process_type: Process::Sequential,
            ..Default::default()
        }
    }

    /// Create a hierarchical process config
    pub fn hierarchical(manager_model: impl Into<String>) -> Self {
        Self {
            process_type: Process::Hierarchical,
            manager_model: Some(manager_model.into()),
            allow_delegation: true,
            ..Default::default()
        }
    }

    /// Create a parallel process config
    pub fn parallel(max_parallel: usize) -> Self {
        Self {
            process_type: Process::Parallel,
            max_parallel,
            ..Default::default()
        }
    }

    /// Set fail-fast behavior
    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Set retry behavior
    pub fn with_retry(mut self, retry: bool, max_retries: usize) -> Self {
        self.retry_failed = retry;
        self.max_retries = max_retries;
        self
    }

    /// Set crew timeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.crew_timeout = Some(seconds);
        self
    }

    /// Enable verbose logging
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }
}

impl Process {
    /// Get a human-readable description of the process
    pub fn description(&self) -> &'static str {
        match self {
            Process::Sequential => "Tasks are executed one after another in order",
            Process::Hierarchical => "A manager agent delegates and coordinates tasks",
            Process::Parallel => "Tasks are executed in parallel where possible",
            Process::Custom => "Custom execution logic defined by the user",
        }
    }

    /// Check if this process requires a manager agent
    pub fn requires_manager(&self) -> bool {
        matches!(self, Process::Hierarchical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_defaults() {
        let config = ProcessConfig::default();
        assert_eq!(config.process_type, Process::Sequential);
        assert!(config.allow_delegation);
        assert!(!config.fail_fast);
    }

    #[test]
    fn test_hierarchical_config() {
        let config = ProcessConfig::hierarchical("gpt-4");
        assert_eq!(config.process_type, Process::Hierarchical);
        assert_eq!(config.manager_model, Some("gpt-4".to_string()));
    }

    #[test]
    fn test_parallel_config() {
        let config = ProcessConfig::parallel(8);
        assert_eq!(config.process_type, Process::Parallel);
        assert_eq!(config.max_parallel, 8);
    }
}
