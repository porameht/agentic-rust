//! Flow System for Event-Driven Workflows
//!
//! Flows provide fine-grained control over complex automations through
//! event-driven workflows. Unlike Crews which focus on agent collaboration,
//! Flows handle precise execution paths and state management.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use uuid::Uuid;

use super::crew::{Crew, CrewError, CrewResult};

/// Errors that can occur in flow execution
#[derive(Error, Debug)]
pub enum FlowError {
    #[error("Flow execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid state transition from {0} to {1}")]
    InvalidTransition(String, String),

    #[error("State not found: {0}")]
    StateNotFound(String),

    #[error("Crew error: {0}")]
    CrewError(#[from] CrewError),

    #[error("Flow timeout after {0} seconds")]
    Timeout(u64),

    #[error("Condition evaluation failed: {0}")]
    ConditionError(String),

    #[error("Maximum iterations exceeded: {0}")]
    MaxIterationsExceeded(usize),
}

/// State of a flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowState {
    /// State identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Description of this state
    pub description: Option<String>,

    /// Whether this is the initial state
    pub is_initial: bool,

    /// Whether this is a final state
    pub is_final: bool,

    /// Crew to execute when entering this state
    pub crew_id: Option<String>,

    /// Timeout for this state in seconds
    pub timeout: Option<u64>,

    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl FlowState {
    /// Create a new flow state
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            is_initial: false,
            is_final: false,
            crew_id: None,
            timeout: None,
            metadata: HashMap::new(),
        }
    }

    /// Set as initial state
    pub fn initial(mut self) -> Self {
        self.is_initial = true;
        self
    }

    /// Set as final state
    pub fn final_state(mut self) -> Self {
        self.is_final = true;
        self
    }

    /// Set the crew to execute
    pub fn with_crew(mut self, crew_id: impl Into<String>) -> Self {
        self.crew_id = Some(crew_id.into());
        self
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout = Some(seconds);
        self
    }
}

/// Condition type for transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionCondition {
    /// Always transition
    Always,

    /// Transition if the previous result was successful
    OnSuccess,

    /// Transition if the previous result failed
    OnFailure,

    /// Transition based on output content containing a string
    OutputContains(String),

    /// Transition based on output matching a regex pattern
    OutputMatches(String),

    /// Transition based on a custom expression
    Expression(String),

    /// Transition based on variable value
    VariableEquals { name: String, value: serde_json::Value },

    /// Combine multiple conditions with AND
    And(Vec<TransitionCondition>),

    /// Combine multiple conditions with OR
    Or(Vec<TransitionCondition>),

    /// Negate a condition
    Not(Box<TransitionCondition>),
}

impl Default for TransitionCondition {
    fn default() -> Self {
        Self::Always
    }
}

/// State transition definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Unique identifier
    pub id: String,

    /// Source state
    pub from: String,

    /// Target state
    pub to: String,

    /// Condition for this transition
    pub condition: TransitionCondition,

    /// Priority (higher = evaluated first)
    pub priority: i32,

    /// Description
    pub description: Option<String>,
}

impl StateTransition {
    /// Create a new transition
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from: from.into(),
            to: to.into(),
            condition: TransitionCondition::Always,
            priority: 0,
            description: None,
        }
    }

    /// Set condition
    pub fn when(mut self, condition: TransitionCondition) -> Self {
        self.condition = condition;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Context for evaluating conditions
#[derive(Debug, Clone)]
pub struct TransitionContext {
    /// Result from the current state's crew execution
    pub crew_result: Option<CrewResult>,

    /// Flow variables
    pub variables: HashMap<String, serde_json::Value>,

    /// Current state ID
    pub current_state: String,

    /// Execution history
    pub history: Vec<String>,
}

impl TransitionContext {
    /// Check if the last crew execution was successful
    pub fn was_successful(&self) -> bool {
        self.crew_result.as_ref().map(|r| r.success).unwrap_or(false)
    }

    /// Get the output from the last crew execution
    pub fn output(&self) -> Option<&str> {
        self.crew_result.as_ref().map(|r| r.output.as_str())
    }

    /// Get a variable value
    pub fn get_variable(&self, name: &str) -> Option<&serde_json::Value> {
        self.variables.get(name)
    }

    /// Set a variable value
    pub fn set_variable(&mut self, name: impl Into<String>, value: serde_json::Value) {
        self.variables.insert(name.into(), value);
    }
}

/// Result of flow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowResult {
    /// Unique execution ID
    pub execution_id: String,

    /// Final state reached
    pub final_state: String,

    /// All crew results in order
    pub crew_results: Vec<CrewResult>,

    /// Final output
    pub output: String,

    /// Execution statistics
    pub stats: FlowStats,

    /// Whether execution was successful
    pub success: bool,

    /// Error message if failed
    pub error: Option<String>,

    /// State history (path taken)
    pub state_history: Vec<String>,

    /// Final variables
    pub variables: HashMap<String, serde_json::Value>,
}

/// Flow execution statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlowStats {
    /// Total execution time in milliseconds
    pub total_time_ms: u64,

    /// Number of states visited
    pub states_visited: usize,

    /// Number of transitions made
    pub transitions_made: usize,

    /// Number of crews executed
    pub crews_executed: usize,

    /// Timestamp when execution started
    pub started_at: chrono::DateTime<chrono::Utc>,

    /// Timestamp when execution completed
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

/// Event listener for flow execution
#[async_trait]
pub trait FlowEventListener: Send + Sync {
    /// Called when flow execution starts
    async fn on_flow_start(&self, _flow_id: &str) {}

    /// Called when flow execution completes
    async fn on_flow_complete(&self, _flow_id: &str, _result: &FlowResult) {}

    /// Called when entering a state
    async fn on_state_enter(&self, _state_id: &str) {}

    /// Called when exiting a state
    async fn on_state_exit(&self, _state_id: &str) {}

    /// Called when a transition occurs
    async fn on_transition(&self, _from: &str, _to: &str) {}
}

/// A Flow for event-driven workflow execution
pub struct Flow {
    /// Flow identifier
    id: String,

    /// Flow name
    name: String,

    /// Description
    description: Option<String>,

    /// States in the flow
    states: HashMap<String, FlowState>,

    /// Transitions between states
    transitions: Vec<StateTransition>,

    /// Crews available to the flow
    crews: HashMap<String, Crew>,

    /// Event listeners
    listeners: Vec<Arc<dyn FlowEventListener>>,

    /// Maximum iterations (to prevent infinite loops)
    max_iterations: usize,

    /// Flow variables
    variables: RwLock<HashMap<String, serde_json::Value>>,
}

impl Flow {
    /// Create a new flow builder
    pub fn builder() -> FlowBuilder {
        FlowBuilder::new()
    }

    /// Get flow ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get flow name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the initial state
    pub fn initial_state(&self) -> Option<&FlowState> {
        self.states.values().find(|s| s.is_initial)
    }

    /// Validate the flow configuration
    pub fn validate(&self) -> Result<(), FlowError> {
        // Must have an initial state
        if self.initial_state().is_none() {
            return Err(FlowError::ExecutionFailed(
                "No initial state defined".to_string(),
            ));
        }

        // Must have at least one final state
        if !self.states.values().any(|s| s.is_final) {
            return Err(FlowError::ExecutionFailed(
                "No final state defined".to_string(),
            ));
        }

        // All transition states must exist
        for transition in &self.transitions {
            if !self.states.contains_key(&transition.from) {
                return Err(FlowError::StateNotFound(transition.from.clone()));
            }
            if !self.states.contains_key(&transition.to) {
                return Err(FlowError::StateNotFound(transition.to.clone()));
            }
        }

        Ok(())
    }

    /// Evaluate a transition condition
    fn evaluate_condition(
        &self,
        condition: &TransitionCondition,
        context: &TransitionContext,
    ) -> bool {
        match condition {
            TransitionCondition::Always => true,
            TransitionCondition::OnSuccess => context.was_successful(),
            TransitionCondition::OnFailure => !context.was_successful(),
            TransitionCondition::OutputContains(text) => context
                .output()
                .map(|o| o.contains(text))
                .unwrap_or(false),
            TransitionCondition::OutputMatches(pattern) => {
                // Simple pattern matching (in production, use regex)
                context
                    .output()
                    .map(|o| o.contains(pattern))
                    .unwrap_or(false)
            }
            TransitionCondition::Expression(_expr) => {
                // TODO: Implement expression evaluation
                true
            }
            TransitionCondition::VariableEquals { name, value } => context
                .get_variable(name)
                .map(|v| v == value)
                .unwrap_or(false),
            TransitionCondition::And(conditions) => conditions
                .iter()
                .all(|c| self.evaluate_condition(c, context)),
            TransitionCondition::Or(conditions) => conditions
                .iter()
                .any(|c| self.evaluate_condition(c, context)),
            TransitionCondition::Not(condition) => !self.evaluate_condition(condition, context),
        }
    }

    /// Find the next transition from current state
    fn find_next_transition(&self, context: &TransitionContext) -> Option<&StateTransition> {
        let mut valid_transitions: Vec<_> = self
            .transitions
            .iter()
            .filter(|t| t.from == context.current_state)
            .filter(|t| self.evaluate_condition(&t.condition, context))
            .collect();

        // Sort by priority (highest first)
        valid_transitions.sort_by(|a, b| b.priority.cmp(&a.priority));

        valid_transitions.first().copied()
    }

    /// Execute the flow
    pub async fn run(&mut self) -> Result<FlowResult, FlowError> {
        self.validate()?;

        let execution_id = Uuid::new_v4().to_string();
        let started_at = chrono::Utc::now();

        info!(flow_id = %self.id, execution_id = %execution_id, "Starting flow execution");

        // Notify listeners
        for listener in &self.listeners {
            listener.on_flow_start(&self.id).await;
        }

        let initial_state = self
            .initial_state()
            .ok_or_else(|| FlowError::ExecutionFailed("No initial state".to_string()))?
            .id
            .clone();

        let mut context = TransitionContext {
            crew_result: None,
            variables: self.variables.read().await.clone(),
            current_state: initial_state.clone(),
            history: vec![initial_state.clone()],
        };

        let mut crew_results = Vec::new();
        let mut iterations = 0;

        loop {
            iterations += 1;
            if iterations > self.max_iterations {
                return Err(FlowError::MaxIterationsExceeded(self.max_iterations));
            }

            let current_state = self
                .states
                .get(&context.current_state)
                .ok_or_else(|| FlowError::StateNotFound(context.current_state.clone()))?;

            // Notify listeners
            for listener in &self.listeners {
                listener.on_state_enter(&current_state.id).await;
            }

            debug!(state_id = %current_state.id, "Entering state");

            // Execute crew if assigned
            if let Some(crew_id) = &current_state.crew_id {
                if let Some(crew) = self.crews.get_mut(crew_id) {
                    info!(crew_id = %crew_id, "Executing crew for state");
                    let result = crew.kickoff().await?;
                    context.crew_result = Some(result.clone());
                    crew_results.push(result);
                }
            }

            // Notify listeners
            for listener in &self.listeners {
                listener.on_state_exit(&current_state.id).await;
            }

            // Check if we've reached a final state
            if current_state.is_final {
                info!(state_id = %current_state.id, "Reached final state");
                break;
            }

            // Find next transition
            if let Some(transition) = self.find_next_transition(&context) {
                debug!(
                    from = %transition.from,
                    to = %transition.to,
                    "Transitioning"
                );

                // Notify listeners
                for listener in &self.listeners {
                    listener.on_transition(&transition.from, &transition.to).await;
                }

                context.current_state = transition.to.clone();
                context.history.push(transition.to.clone());
            } else {
                // No valid transition found
                return Err(FlowError::ExecutionFailed(format!(
                    "No valid transition from state {}",
                    current_state.id
                )));
            }
        }

        let completed_at = chrono::Utc::now();
        let total_time_ms = (completed_at - started_at).num_milliseconds() as u64;

        // Combine all crew outputs
        let output = crew_results
            .iter()
            .map(|r| r.output.clone())
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        let result = FlowResult {
            execution_id,
            final_state: context.current_state.clone(),
            crew_results,
            output,
            stats: FlowStats {
                total_time_ms,
                states_visited: context.history.len(),
                transitions_made: context.history.len() - 1,
                crews_executed: iterations,
                started_at,
                completed_at,
            },
            success: true,
            error: None,
            state_history: context.history,
            variables: context.variables,
        };

        // Notify listeners
        for listener in &self.listeners {
            listener.on_flow_complete(&self.id, &result).await;
        }

        info!(
            flow_id = %self.id,
            total_time_ms = result.stats.total_time_ms,
            states_visited = result.stats.states_visited,
            "Flow execution completed"
        );

        Ok(result)
    }

    /// Set a flow variable
    pub async fn set_variable(&self, name: impl Into<String>, value: serde_json::Value) {
        let mut vars = self.variables.write().await;
        vars.insert(name.into(), value);
    }

    /// Get a flow variable
    pub async fn get_variable(&self, name: &str) -> Option<serde_json::Value> {
        let vars = self.variables.read().await;
        vars.get(name).cloned()
    }
}

/// Builder for creating flows
pub struct FlowBuilder {
    id: String,
    name: String,
    description: Option<String>,
    states: Vec<FlowState>,
    transitions: Vec<StateTransition>,
    crews: HashMap<String, Crew>,
    listeners: Vec<Arc<dyn FlowEventListener>>,
    max_iterations: usize,
    initial_variables: HashMap<String, serde_json::Value>,
}

impl FlowBuilder {
    /// Create a new flow builder
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: String::new(),
            description: None,
            states: Vec::new(),
            transitions: Vec::new(),
            crews: HashMap::new(),
            listeners: Vec::new(),
            max_iterations: 100,
            initial_variables: HashMap::new(),
        }
    }

    /// Set flow ID
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Set flow name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set flow description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a state
    pub fn state(mut self, state: FlowState) -> Self {
        self.states.push(state);
        self
    }

    /// Add a transition
    pub fn transition(mut self, transition: StateTransition) -> Self {
        self.transitions.push(transition);
        self
    }

    /// Add a simple transition (always transition on completion)
    pub fn simple_transition(self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.transition(StateTransition::new(from, to))
    }

    /// Add a crew
    pub fn crew(mut self, id: impl Into<String>, crew: Crew) -> Self {
        self.crews.insert(id.into(), crew);
        self
    }

    /// Add an event listener
    pub fn listener(mut self, listener: Arc<dyn FlowEventListener>) -> Self {
        self.listeners.push(listener);
        self
    }

    /// Set maximum iterations
    pub fn max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    /// Set an initial variable
    pub fn variable(mut self, name: impl Into<String>, value: serde_json::Value) -> Self {
        self.initial_variables.insert(name.into(), value);
        self
    }

    /// Build the flow
    pub fn build(self) -> Flow {
        let states: HashMap<_, _> = self.states.into_iter().map(|s| (s.id.clone(), s)).collect();

        Flow {
            id: self.id,
            name: self.name,
            description: self.description,
            states,
            transitions: self.transitions,
            crews: self.crews,
            listeners: self.listeners,
            max_iterations: self.max_iterations,
            variables: RwLock::new(self.initial_variables),
        }
    }
}

impl Default for FlowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_state_builder() {
        let state = FlowState::new("start", "Start State")
            .initial()
            .with_description("The beginning")
            .with_crew("init-crew");

        assert_eq!(state.id, "start");
        assert!(state.is_initial);
        assert!(!state.is_final);
        assert_eq!(state.crew_id, Some("init-crew".to_string()));
    }

    #[test]
    fn test_transition_conditions() {
        let context = TransitionContext {
            crew_result: Some(CrewResult {
                execution_id: "test".to_string(),
                output: "success: task completed".to_string(),
                task_outputs: HashMap::new(),
                stats: Default::default(),
                success: true,
                error: None,
                raw_outputs: Vec::new(),
            }),
            variables: HashMap::new(),
            current_state: "test".to_string(),
            history: Vec::new(),
        };

        let flow = Flow::builder().build();

        // Test Always condition
        assert!(flow.evaluate_condition(&TransitionCondition::Always, &context));

        // Test OnSuccess condition
        assert!(flow.evaluate_condition(&TransitionCondition::OnSuccess, &context));
        assert!(!flow.evaluate_condition(&TransitionCondition::OnFailure, &context));

        // Test OutputContains
        assert!(flow.evaluate_condition(
            &TransitionCondition::OutputContains("success".to_string()),
            &context
        ));
    }

    #[test]
    fn test_flow_validation() {
        // Flow without initial state
        let flow = Flow::builder()
            .state(FlowState::new("end", "End").final_state())
            .build();

        assert!(flow.validate().is_err());

        // Flow without final state
        let flow = Flow::builder()
            .state(FlowState::new("start", "Start").initial())
            .build();

        assert!(flow.validate().is_err());

        // Valid flow
        let flow = Flow::builder()
            .state(FlowState::new("start", "Start").initial())
            .state(FlowState::new("end", "End").final_state())
            .simple_transition("start", "end")
            .build();

        assert!(flow.validate().is_ok());
    }
}
