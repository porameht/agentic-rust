//! Integration tests for CrewAI-style multi-agent architecture
//!
//! These tests verify that the crew system works end-to-end,
//! including agent execution, task orchestration, and flow management.

#[cfg(test)]
mod integration_tests {
    use crate::crew::agent::AgentExecutor;
    use crate::crew::{
        Agent, Crew, CrewResult, Flow, FlowState, Memory, MemoryConfig, MemoryType, Process,
        ProcessConfig, StateTransition, Task, TaskOutput, TaskStatus, TransitionCondition,
    };
    use std::collections::HashMap;
    use std::sync::Arc;

    // ==================== Agent Tests ====================

    #[test]
    fn test_agent_creation_with_full_config() {
        let agent = Agent::builder()
            .id("test-agent-001")
            .role("Senior Software Engineer")
            .goal("Write high-quality, maintainable code")
            .backstory(
                "A seasoned developer with 15 years of experience in Rust and distributed systems",
            )
            .model("gpt-4")
            .temperature(0.5)
            .max_tokens(4096)
            .max_iterations(5)
            .max_execution_time(120)
            .verbose(true)
            .allow_delegation(true)
            .tool_name("code_search")
            .tool_name("file_editor")
            .system_prompt_suffix("Always follow best practices.")
            .response_format("markdown")
            .metadata("department", serde_json::json!("engineering"))
            .build();

        assert_eq!(agent.id(), "test-agent-001");
        assert_eq!(agent.role(), "Senior Software Engineer");
        assert_eq!(agent.goal(), "Write high-quality, maintainable code");
        assert!(agent.backstory().contains("15 years"));
        assert_eq!(agent.model(), "gpt-4");
        assert!(agent.allows_delegation());
    }

    #[test]
    fn test_agent_system_prompt_generation() {
        let agent = Agent::builder()
            .role("Data Analyst")
            .goal("Analyze data and provide insights")
            .backstory("Expert in statistical analysis and data visualization")
            .tool_name("data_query")
            .tool_name("chart_generator")
            .system_prompt_suffix("Use clear visualizations when possible.")
            .build();

        let prompt = agent.system_prompt();

        assert!(prompt.contains("Data Analyst"));
        assert!(prompt.contains("Analyze data and provide insights"));
        assert!(prompt.contains("statistical analysis"));
        assert!(prompt.contains("data_query"));
        assert!(prompt.contains("chart_generator"));
        assert!(prompt.contains("Use clear visualizations"));
    }

    #[tokio::test]
    async fn test_agent_execution() {
        use crate::crew::agent::{AgentExecutor, ExecutionContext};

        let agent = Agent::builder()
            .id("executor-agent")
            .role("Task Executor")
            .goal("Execute tasks efficiently")
            .backstory("Reliable executor with attention to detail")
            .verbose(true)
            .build();

        let context = ExecutionContext {
            task_description: "Analyze the provided data and summarize findings".to_string(),
            expected_output: "A concise summary with key insights".to_string(),
            context: vec!["Previous analysis showed growth trend".to_string()],
            available_tools: vec![],
            shared_state: HashMap::new(),
            iteration: 0,
            max_iterations: 10,
        };

        let result = agent.execute(context).await;

        assert!(result.is_ok());
        let execution_result = result.unwrap();
        assert!(!execution_result.output.is_empty());
        assert!(execution_result.metadata.iterations > 0);
        assert!(execution_result.reasoning.is_some()); // verbose mode enabled
    }

    // ==================== Task Tests ====================

    #[test]
    fn test_task_creation_and_lifecycle() {
        let mut task = Task::builder()
            .id("task-lifecycle-test")
            .name("Lifecycle Test Task")
            .description("Test task lifecycle management")
            .expected_output("Successful lifecycle completion")
            .agent("test-agent")
            .timeout(60)
            .max_retries(3)
            .build();

        // Initial state
        assert_eq!(task.status(), TaskStatus::Pending);
        assert!(task.output().is_none());

        // Start task
        task.start();
        assert_eq!(task.status(), TaskStatus::InProgress);

        // Complete task
        let output = TaskOutput::new("Task completed successfully")
            .with_summary("Quick summary")
            .with_note("Executed without issues");
        task.complete(output);

        assert_eq!(task.status(), TaskStatus::Completed);
        assert!(task.output().is_some());
        assert_eq!(
            task.output().unwrap().result,
            "Task completed successfully"
        );
    }

    #[test]
    fn test_task_dependencies_and_context() {
        let mut task = Task::builder()
            .id("dependent-task")
            .description("Task with dependencies")
            .expected_output("Combined output")
            .agent("worker")
            .depends_on("task-1")
            .depends_on("task-2")
            .context_instructions("Combine insights from both previous tasks")
            .build();

        assert_eq!(task.dependencies().len(), 2);

        // Add context from completed dependencies
        task.add_context("task-1".to_string(), "Output from task 1".to_string(), true);
        task.add_context("task-2".to_string(), "Output from task 2".to_string(), true);

        assert_eq!(task.context().len(), 2);

        // Build prompt includes context
        let prompt = task.build_prompt();
        assert!(prompt.contains("Output from task 1"));
        assert!(prompt.contains("Output from task 2"));
        assert!(prompt.contains("Combine insights"));
    }

    #[test]
    fn test_task_failure_and_retry() {
        let mut task = Task::builder()
            .id("retry-task")
            .description("Task that may fail")
            .expected_output("Success after retry")
            .agent("worker")
            .max_retries(2)
            .build();

        // First attempt fails
        task.start();
        task.fail("Network error");

        assert_eq!(task.status(), TaskStatus::Failed);
        assert!(task.can_retry()); // Still has retries left

        // Reset for retry
        task.reset();
        assert_eq!(task.status(), TaskStatus::Pending);

        // Second attempt
        task.start();
        task.complete(TaskOutput::new("Success!"));
        assert_eq!(task.status(), TaskStatus::Completed);
    }

    #[test]
    fn test_task_is_ready_with_dependencies() {
        let task = Task::builder()
            .id("check-ready")
            .description("Check readiness")
            .expected_output("Ready check result")
            .agent("worker")
            .depends_on("dep-1")
            .depends_on("dep-2")
            .build();

        let mut completed: HashMap<String, TaskOutput> = HashMap::new();

        // Not ready - no dependencies completed
        assert!(!task.is_ready(&completed));

        // Not ready - only one dependency completed
        completed.insert("dep-1".to_string(), TaskOutput::new("Done"));
        assert!(!task.is_ready(&completed));

        // Ready - all dependencies completed
        completed.insert("dep-2".to_string(), TaskOutput::new("Done"));
        assert!(task.is_ready(&completed));
    }

    // ==================== Crew Tests ====================

    #[test]
    fn test_crew_creation_and_validation() {
        let agent1 = Agent::builder()
            .id("agent-1")
            .role("Researcher")
            .goal("Research topics")
            .backstory("Expert researcher")
            .build();

        let agent2 = Agent::builder()
            .id("agent-2")
            .role("Writer")
            .goal("Write content")
            .backstory("Expert writer")
            .build();

        let task1 = Task::builder()
            .id("research")
            .description("Research the topic")
            .expected_output("Research findings")
            .agent("agent-1")
            .build();

        let task2 = Task::builder()
            .id("write")
            .description("Write based on research")
            .expected_output("Written content")
            .agent("agent-2")
            .depends_on("research")
            .build();

        let crew = Crew::builder()
            .id("research-write-crew")
            .name("Research and Write Crew")
            .description("Crew that researches and writes")
            .agent(agent1)
            .agent(agent2)
            .task(task1)
            .task(task2)
            .process(Process::Sequential)
            .verbose(true)
            .build();

        assert_eq!(crew.id(), "research-write-crew");
        assert_eq!(crew.agents().len(), 2);
        assert_eq!(crew.tasks().len(), 2);
        assert!(crew.validate().is_ok());
    }

    #[test]
    fn test_crew_validation_fails_without_agents() {
        let task = Task::builder()
            .id("orphan-task")
            .description("Task without agent")
            .expected_output("Nothing")
            .agent("non-existent")
            .build();

        let crew = Crew::builder().task(task).build();

        assert!(crew.validate().is_err());
    }

    #[test]
    fn test_crew_validation_fails_without_tasks() {
        let agent = Agent::builder()
            .id("lonely-agent")
            .role("Worker")
            .goal("Work")
            .backstory("Worker")
            .build();

        let crew = Crew::builder().agent(agent).build();

        assert!(crew.validate().is_err());
    }

    #[test]
    fn test_crew_detects_missing_agent() {
        let agent = Agent::builder()
            .id("existing-agent")
            .role("Worker")
            .goal("Work")
            .backstory("Worker")
            .build();

        let task = Task::builder()
            .id("task")
            .description("Do work")
            .expected_output("Work done")
            .agent("non-existing-agent") // Wrong agent ID
            .build();

        let crew = Crew::builder().agent(agent).task(task).build();

        let validation = crew.validate();
        assert!(validation.is_err());
    }

    #[tokio::test]
    async fn test_crew_sequential_execution() {
        let agent = Agent::builder()
            .id("worker")
            .role("Worker")
            .goal("Complete tasks")
            .backstory("Reliable worker")
            .build();

        // Single task without dependencies - simpler test case
        let task1 = Task::builder()
            .id("task-1")
            .description("First task")
            .expected_output("First output")
            .agent("worker")
            .build();

        let mut crew = Crew::builder()
            .id("sequential-crew")
            .agent(agent)
            .task(task1)
            .process(Process::Sequential)
            .build();

        let result = crew.kickoff().await;

        assert!(result.is_ok(), "Crew kickoff failed: {:?}", result.err());
        let crew_result = result.unwrap();
        assert!(
            crew_result.success,
            "Crew execution failed: {:?}",
            crew_result.error
        );
        assert!(
            !crew_result.task_outputs.is_empty(),
            "Expected task outputs but got none"
        );
        assert!(crew_result.task_outputs.contains_key("task-1"));
    }

    #[tokio::test]
    async fn test_crew_with_multiple_agents() {
        let researcher = Agent::builder()
            .id("researcher")
            .role("Researcher")
            .goal("Research topics thoroughly")
            .backstory("Expert researcher")
            .build();

        let analyst = Agent::builder()
            .id("analyst")
            .role("Analyst")
            .goal("Analyze research findings")
            .backstory("Expert analyst")
            .build();

        let writer = Agent::builder()
            .id("writer")
            .role("Writer")
            .goal("Write compelling content")
            .backstory("Expert writer")
            .build();

        let research_task = Task::builder()
            .id("research")
            .description("Research the topic")
            .expected_output("Research data")
            .agent("researcher")
            .build();

        let analysis_task = Task::builder()
            .id("analyze")
            .description("Analyze research")
            .expected_output("Analysis report")
            .agent("analyst")
            .depends_on("research")
            .build();

        let writing_task = Task::builder()
            .id("write")
            .description("Write final report")
            .expected_output("Final report")
            .agent("writer")
            .depends_on("analyze")
            .build();

        let mut crew = Crew::builder()
            .id("multi-agent-crew")
            .agent(researcher)
            .agent(analyst)
            .agent(writer)
            .task(research_task)
            .task(analysis_task)
            .task(writing_task)
            .process(Process::Sequential)
            .build();

        let result = crew.kickoff().await.unwrap();

        assert!(result.success);
        assert_eq!(result.task_outputs.len(), 3);
        assert!(result.output.contains("research"));
    }

    // ==================== Memory Tests ====================

    #[tokio::test]
    async fn test_memory_store_and_retrieve() {
        let memory = Memory::new(MemoryConfig {
            memory_type: MemoryType::ShortTerm,
            max_items: 100,
            ..Default::default()
        });

        // Store values
        memory
            .store("user_name", serde_json::json!("Alice"))
            .await
            .unwrap();
        memory
            .store(
                "preferences",
                serde_json::json!({"theme": "dark", "language": "en"}),
            )
            .await
            .unwrap();

        // Retrieve values
        let name = memory.retrieve("user_name").await.unwrap();
        assert_eq!(name, Some(serde_json::json!("Alice")));

        let prefs = memory.retrieve("preferences").await.unwrap();
        assert!(prefs.is_some());
        assert_eq!(prefs.unwrap()["theme"], "dark");

        // Non-existent key
        let missing = memory.retrieve("non_existent").await.unwrap();
        assert!(missing.is_none());
    }

    #[tokio::test]
    async fn test_memory_search() {
        let memory = Memory::new(MemoryConfig::default());

        memory
            .store("customer_alice", serde_json::json!({"name": "Alice", "type": "premium"}))
            .await
            .unwrap();
        memory
            .store("customer_bob", serde_json::json!({"name": "Bob", "type": "standard"}))
            .await
            .unwrap();
        memory
            .store("product_widget", serde_json::json!({"name": "Widget", "price": 99}))
            .await
            .unwrap();

        // Search for customers
        let results = memory.search("customer", 10).await.unwrap();
        assert_eq!(results.len(), 2);

        // Search for specific name
        let alice_results = memory.search("Alice", 10).await.unwrap();
        assert_eq!(alice_results.len(), 1);
    }

    #[tokio::test]
    async fn test_memory_capacity_limit() {
        let memory = Memory::new(MemoryConfig {
            max_items: 3,
            ..Default::default()
        });

        // Fill memory
        memory.store("item1", serde_json::json!(1)).await.unwrap();
        memory.store("item2", serde_json::json!(2)).await.unwrap();
        memory.store("item3", serde_json::json!(3)).await.unwrap();

        assert_eq!(memory.len().await.unwrap(), 3);

        // Add one more - should evict oldest
        memory.store("item4", serde_json::json!(4)).await.unwrap();

        assert_eq!(memory.len().await.unwrap(), 3);
        // item1 should be evicted
        assert!(memory.retrieve("item4").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_agent_with_memory() {
        let agent = Agent::builder()
            .id("memory-agent")
            .role("Memory Agent")
            .goal("Remember things")
            .backstory("Agent with good memory")
            .with_short_term_memory()
            .build();

        // Store something in agent's memory
        agent
            .remember("last_action", serde_json::json!("searched for documents"))
            .await
            .unwrap();

        // Recall it
        let recalled = agent.recall("last_action").await.unwrap();
        assert!(recalled.is_some());
        assert_eq!(recalled.unwrap(), "searched for documents");
    }

    // ==================== Flow Tests ====================

    #[test]
    fn test_flow_state_creation() {
        let state = FlowState::new("processing", "Processing State")
            .with_description("Handles data processing")
            .with_crew("processing-crew")
            .with_timeout(300);

        assert_eq!(state.id, "processing");
        assert_eq!(state.name, "Processing State");
        assert!(!state.is_initial);
        assert!(!state.is_final);
        assert_eq!(state.crew_id, Some("processing-crew".to_string()));
        assert_eq!(state.timeout, Some(300));
    }

    #[test]
    fn test_flow_transition_conditions() {
        use crate::crew::crew::CrewResult;
        use crate::crew::flow::TransitionContext;

        let context_success = TransitionContext {
            crew_result: Some(CrewResult {
                execution_id: "test".to_string(),
                output: "approved: content looks good".to_string(),
                task_outputs: HashMap::new(),
                stats: Default::default(),
                success: true,
                error: None,
                raw_outputs: vec![],
            }),
            variables: HashMap::new(),
            current_state: "review".to_string(),
            history: vec![],
        };

        let context_failure = TransitionContext {
            crew_result: Some(CrewResult {
                execution_id: "test".to_string(),
                output: "revision needed: please fix errors".to_string(),
                task_outputs: HashMap::new(),
                stats: Default::default(),
                success: false,
                error: Some("Validation failed".to_string()),
                raw_outputs: vec![],
            }),
            variables: HashMap::new(),
            current_state: "review".to_string(),
            history: vec![],
        };

        // Test condition evaluation via flow
        let flow = Flow::builder()
            .state(FlowState::new("start", "Start").initial())
            .state(FlowState::new("end", "End").final_state())
            .simple_transition("start", "end")
            .build();

        // OnSuccess condition
        assert!(context_success.was_successful());
        assert!(!context_failure.was_successful());

        // OutputContains
        assert!(context_success.output().unwrap().contains("approved"));
        assert!(context_failure.output().unwrap().contains("revision"));
    }

    #[test]
    fn test_flow_validation() {
        // Valid flow
        let valid_flow = Flow::builder()
            .id("valid-flow")
            .name("Valid Flow")
            .state(FlowState::new("start", "Start").initial())
            .state(FlowState::new("middle", "Middle"))
            .state(FlowState::new("end", "End").final_state())
            .simple_transition("start", "middle")
            .simple_transition("middle", "end")
            .build();

        assert!(valid_flow.validate().is_ok());

        // Invalid flow - no initial state
        let no_initial = Flow::builder()
            .state(FlowState::new("end", "End").final_state())
            .build();

        assert!(no_initial.validate().is_err());

        // Invalid flow - no final state
        let no_final = Flow::builder()
            .state(FlowState::new("start", "Start").initial())
            .build();

        assert!(no_final.validate().is_err());
    }

    #[test]
    fn test_flow_with_conditional_transitions() {
        let _flow = Flow::builder()
            .id("conditional-flow")
            .state(FlowState::new("review", "Review").initial())
            .state(FlowState::new("approved", "Approved").final_state())
            .state(FlowState::new("rejected", "Rejected").final_state())
            .transition(
                StateTransition::new("review", "approved")
                    .when(TransitionCondition::OutputContains("approved".to_string()))
                    .with_priority(10),
            )
            .transition(
                StateTransition::new("review", "rejected")
                    .when(TransitionCondition::OutputContains("rejected".to_string()))
                    .with_priority(5),
            )
            .build();

        assert!(flow.validate().is_ok());
    }

    #[tokio::test]
    async fn test_flow_variable_management() {
        let flow = Flow::builder()
            .id("var-flow")
            .state(FlowState::new("start", "Start").initial())
            .state(FlowState::new("end", "End").final_state())
            .simple_transition("start", "end")
            .variable("counter", serde_json::json!(0))
            .variable("name", serde_json::json!("test"))
            .build();

        // Get initial variables
        let counter = flow.get_variable("counter").await;
        assert_eq!(counter, Some(serde_json::json!(0)));

        // Set new variable
        flow.set_variable("status", serde_json::json!("running"))
            .await;
        let status = flow.get_variable("status").await;
        assert_eq!(status, Some(serde_json::json!("running")));
    }

    // ==================== Process Configuration Tests ====================

    #[test]
    fn test_process_configurations() {
        // Sequential config
        let sequential = ProcessConfig::sequential();
        assert_eq!(sequential.process_type, Process::Sequential);

        // Hierarchical config
        let hierarchical = ProcessConfig::hierarchical("gpt-4");
        assert_eq!(hierarchical.process_type, Process::Hierarchical);
        assert_eq!(hierarchical.manager_model, Some("gpt-4".to_string()));

        // Parallel config
        let parallel = ProcessConfig::parallel(8);
        assert_eq!(parallel.process_type, Process::Parallel);
        assert_eq!(parallel.max_parallel, 8);

        // Config with options
        let config = ProcessConfig::sequential()
            .with_fail_fast(true)
            .with_retry(true, 3)
            .with_timeout(600)
            .verbose();

        assert!(config.fail_fast);
        assert!(config.retry_failed);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.crew_timeout, Some(600));
        assert!(config.verbose);
    }

    // ==================== Example Crews Tests ====================

    #[tokio::test]
    async fn test_research_crew_execution() {
        use crate::crew::examples::create_research_crew;

        let mut crew = create_research_crew("Rust programming", "technical article");

        assert_eq!(crew.agents().len(), 2);
        assert_eq!(crew.tasks().len(), 2);
        assert!(crew.validate().is_ok());

        let result = crew.kickoff().await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_sales_crew_execution() {
        use crate::crew::examples::create_sales_crew;

        let mut crew = create_sales_crew("Cloud Services", "English");

        assert_eq!(crew.agents().len(), 3);
        assert_eq!(crew.tasks().len(), 3);
        assert!(crew.validate().is_ok());

        let result = crew.kickoff().await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_code_review_crew_execution() {
        use crate::crew::examples::create_code_review_crew;

        let mut crew = create_code_review_crew("Rust");

        assert_eq!(crew.agents().len(), 3);
        assert_eq!(crew.tasks().len(), 4);
        assert!(crew.validate().is_ok());

        let result = crew.kickoff().await.unwrap();
        assert!(result.success);
        assert_eq!(result.task_outputs.len(), 4);
    }

    #[test]
    fn test_content_flow_structure() {
        use crate::crew::examples::create_content_flow;

        let flow = create_content_flow();

        assert_eq!(flow.id(), "content-flow");
        assert!(flow.validate().is_ok());
        assert!(flow.initial_state().is_some());
        assert_eq!(flow.initial_state().unwrap().id, "research");
    }

    #[test]
    fn test_support_flow_structure() {
        use crate::crew::examples::create_support_flow;

        let flow = create_support_flow();

        assert_eq!(flow.id(), "support-flow");
        assert!(flow.validate().is_ok());
        assert!(flow.initial_state().is_some());
        assert_eq!(flow.initial_state().unwrap().id, "triage");
    }

    // ==================== Event Listener Tests ====================

    use crate::crew::crew::CrewEventListener;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestEventListener {
        crew_starts: AtomicUsize,
        crew_completes: AtomicUsize,
        task_starts: AtomicUsize,
        task_completes: AtomicUsize,
    }

    impl TestEventListener {
        fn new() -> Self {
            Self {
                crew_starts: AtomicUsize::new(0),
                crew_completes: AtomicUsize::new(0),
                task_starts: AtomicUsize::new(0),
                task_completes: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl CrewEventListener for TestEventListener {
        async fn on_crew_start(&self, _crew_id: &str) {
            self.crew_starts.fetch_add(1, Ordering::SeqCst);
        }

        async fn on_crew_complete(&self, _crew_id: &str, _result: &CrewResult) {
            self.crew_completes.fetch_add(1, Ordering::SeqCst);
        }

        async fn on_task_start(&self, _task_id: &str, _agent_id: &str) {
            self.task_starts.fetch_add(1, Ordering::SeqCst);
        }

        async fn on_task_complete(&self, _task_id: &str, _output: &TaskOutput) {
            self.task_completes.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[tokio::test]
    async fn test_crew_event_listeners() {
        let listener = Arc::new(TestEventListener::new());

        let agent = Agent::builder()
            .id("worker")
            .role("Worker")
            .goal("Work")
            .backstory("Worker")
            .build();

        let task1 = Task::builder()
            .id("task-1")
            .description("Task 1")
            .expected_output("Output 1")
            .agent("worker")
            .build();

        let task2 = Task::builder()
            .id("task-2")
            .description("Task 2")
            .expected_output("Output 2")
            .agent("worker")
            .build();

        let mut crew = Crew::builder()
            .agent(agent)
            .task(task1)
            .task(task2)
            .listener(listener.clone())
            .build();

        crew.kickoff().await.unwrap();

        assert_eq!(listener.crew_starts.load(Ordering::SeqCst), 1);
        assert_eq!(listener.crew_completes.load(Ordering::SeqCst), 1);
        assert_eq!(listener.task_starts.load(Ordering::SeqCst), 2);
        assert_eq!(listener.task_completes.load(Ordering::SeqCst), 2);
    }

    // ==================== Complex Scenarios ====================

    #[tokio::test]
    async fn test_crew_with_task_skipping() {
        // Create a scenario where some tasks should be skipped due to unmet dependencies

        let agent = Agent::builder()
            .id("worker")
            .role("Worker")
            .goal("Work")
            .backstory("Worker")
            .build();

        // Task 2 depends on non-existent task
        let task1 = Task::builder()
            .id("task-1")
            .description("First task")
            .expected_output("First output")
            .agent("worker")
            .build();

        let task2 = Task::builder()
            .id("task-2")
            .description("Second task - depends on missing task")
            .expected_output("Second output")
            .agent("worker")
            .depends_on("non-existent-task")
            .build();

        let mut crew = Crew::builder().agent(agent).task(task1).task(task2).build();

        let result = crew.kickoff().await.unwrap();

        // Crew should succeed but task-2 should be skipped
        assert!(result.success);
        // Only task-1 should have output
        assert!(result.task_outputs.contains_key("task-1"));
    }

    #[tokio::test]
    async fn test_end_to_end_content_creation() {
        // Simulate a complete content creation workflow

        let researcher = Agent::builder()
            .id("researcher")
            .role("Senior Research Analyst")
            .goal("Conduct comprehensive research")
            .backstory("Expert researcher with deep industry knowledge")
            .model("gpt-4")
            .temperature(0.3)
            .build();

        let writer = Agent::builder()
            .id("writer")
            .role("Content Writer")
            .goal("Create engaging content")
            .backstory("Award-winning content creator")
            .model("gpt-4")
            .temperature(0.7)
            .build();

        let editor = Agent::builder()
            .id("editor")
            .role("Editor")
            .goal("Polish and refine content")
            .backstory("Experienced editor with attention to detail")
            .model("gpt-4")
            .temperature(0.2)
            .build();

        let research = Task::builder()
            .id("research")
            .name("Research Phase")
            .description("Research the topic of AI agents in Rust")
            .expected_output("Comprehensive research report with key findings")
            .agent("researcher")
            .build();

        let writing = Task::builder()
            .id("writing")
            .name("Writing Phase")
            .description("Write a blog post based on research")
            .expected_output("Draft blog post of 1500 words")
            .agent("writer")
            .depends_on("research")
            .build();

        let editing = Task::builder()
            .id("editing")
            .name("Editing Phase")
            .description("Edit and polish the draft")
            .expected_output("Final polished blog post")
            .agent("editor")
            .depends_on("writing")
            .build();

        let mut crew = Crew::builder()
            .id("content-creation-crew")
            .name("Content Creation Team")
            .agent(researcher)
            .agent(writer)
            .agent(editor)
            .task(research)
            .task(writing)
            .task(editing)
            .process(Process::Sequential)
            .verbose(true)
            .build();

        let result = crew.kickoff().await.unwrap();

        assert!(result.success);
        assert_eq!(result.task_outputs.len(), 3);
        assert!(result.stats.tasks_succeeded >= 3);
        assert!(!result.output.is_empty());
    }
}
