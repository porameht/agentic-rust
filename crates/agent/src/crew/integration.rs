//! Full Integration Example for CrewAI Architecture
//!
//! This module demonstrates a complete end-to-end workflow where:
//! - Multiple agents collaborate on tasks
//! - Tasks pass context to dependent tasks
//! - Crews orchestrate agent execution
//! - Flows manage complex multi-crew workflows
//! - Memory persists context across interactions
//!
//! # Example Workflow: Blog Content Creation Pipeline
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                    CONTENT CREATION PIPELINE                            │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │                                                                         │
//! │  FLOW: content_pipeline_flow                                            │
//! │  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐               │
//! │  │  Research   │────▶│   Writing   │────▶│   Review    │               │
//! │  │   State     │     │   State     │     │   State     │               │
//! │  │ (research   │     │ (writing    │     │ (review     │               │
//! │  │   crew)     │     │   crew)     │     │   crew)     │               │
//! │  └─────────────┘     └─────────────┘     └─────────────┘               │
//! │                                                │                        │
//! │                         ┌──────────────────────┼──────────────────┐    │
//! │                         │                      │                  │    │
//! │                         ▼                      ▼                  │    │
//! │                  ┌─────────────┐       ┌─────────────┐           │    │
//! │                  │  Published  │       │  Revision   │───────────┘    │
//! │                  │   (final)   │       │   State     │                │
//! │                  └─────────────┘       └─────────────┘                │
//! │                                                                        │
//! │  Each crew contains:                                                   │
//! │  ┌─────────────────────────────────────────────────────────────────┐  │
//! │  │ CREW                                                             │  │
//! │  │  Agents: [Agent1, Agent2, ...]                                  │  │
//! │  │  Tasks:  [Task1 ──▶ Task2 ──▶ Task3]  (with dependencies)       │  │
//! │  │  Memory: SharedCrewMemory                                        │  │
//! │  └─────────────────────────────────────────────────────────────────┘  │
//! │                                                                        │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```

use super::agent::Agent;
use super::crew::{Crew, CrewEventListener, CrewResult};
use super::flow::{Flow, FlowEventListener, FlowResult, FlowState, StateTransition, TransitionCondition};
use super::memory::{Memory, MemoryConfig, MemoryType};
use super::process::Process;
use super::task::{Task, TaskOutput};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// COMPLETE PIPELINE EXAMPLE
// ============================================================================

/// Result of the complete pipeline execution
#[derive(Debug, Clone)]
pub struct PipelineResult {
    /// Final output from the pipeline
    pub final_output: String,
    /// All crew results in execution order
    pub crew_results: Vec<CrewResult>,
    /// Total execution time in milliseconds
    pub total_time_ms: u64,
    /// Pipeline statistics
    pub stats: PipelineStats,
}

/// Statistics for pipeline execution
#[derive(Debug, Clone, Default)]
pub struct PipelineStats {
    pub total_agents: usize,
    pub total_tasks: usize,
    pub tasks_completed: usize,
    pub crews_executed: usize,
    pub states_visited: usize,
}

/// Creates a complete content creation pipeline with all components working together
///
/// This example demonstrates:
/// 1. Creating specialized agents with distinct roles
/// 2. Defining tasks with dependencies
/// 3. Building crews that orchestrate agents
/// 4. Using flows to manage multi-crew workflows
/// 5. Shared memory for context persistence
///
/// # Example
///
/// ```rust,ignore
/// use agent::crew::integration::{create_content_pipeline, run_content_pipeline};
///
/// #[tokio::main]
/// async fn main() {
///     let topic = "AI Agent Frameworks in Rust";
///     let result = run_content_pipeline(topic).await.unwrap();
///     println!("Final content: {}", result.final_output);
/// }
/// ```
pub fn create_content_pipeline(topic: &str) -> ContentPipeline {
    ContentPipeline::new(topic)
}

/// A complete content creation pipeline
pub struct ContentPipeline {
    topic: String,
    shared_memory: Arc<Memory>,
    research_crew: Option<Crew>,
    writing_crew: Option<Crew>,
    review_crew: Option<Crew>,
    revision_crew: Option<Crew>,
}

impl ContentPipeline {
    /// Create a new content pipeline for a given topic
    pub fn new(topic: &str) -> Self {
        let shared_memory = Arc::new(Memory::new(MemoryConfig {
            memory_type: MemoryType::ShortTerm,
            max_items: 1000,
            use_embeddings: false,
            persist: false,
            ..Default::default()
        }));

        let mut pipeline = Self {
            topic: topic.to_string(),
            shared_memory,
            research_crew: None,
            writing_crew: None,
            review_crew: None,
            revision_crew: None,
        };

        pipeline.build_research_crew();
        pipeline.build_writing_crew();
        pipeline.build_review_crew();
        pipeline.build_revision_crew();

        pipeline
    }

    /// Build the research crew
    fn build_research_crew(&mut self) {
        // === AGENTS ===
        let topic_researcher = Agent::builder()
            .id("topic-researcher")
            .role("Topic Research Specialist")
            .goal(format!("Conduct comprehensive research on '{}'", self.topic))
            .backstory(
                "You are an expert researcher with 15 years of experience in technology research. \
                 You have a talent for finding relevant information, identifying key trends, \
                 and synthesizing complex topics into clear insights."
            )
            .model("gpt-4")
            .temperature(0.3)
            .verbose(true)
            .with_long_term_memory()
            .tool_name("web_search")
            .tool_name("document_reader")
            .build();

        let data_analyst = Agent::builder()
            .id("data-analyst")
            .role("Data Analyst")
            .goal("Analyze research data and extract key statistics and trends")
            .backstory(
                "You are a data analyst who excels at finding patterns in information. \
                 You can identify trends, compare options, and provide data-driven insights."
            )
            .model("gpt-4")
            .temperature(0.2)
            .tool_name("data_analysis")
            .build();

        // === TASKS ===
        let research_task = Task::builder()
            .id("initial-research")
            .name("Initial Topic Research")
            .description(format!(
                "Conduct thorough research on '{}'. \
                 Gather information from multiple sources including:\n\
                 - Current state of the technology\n\
                 - Key players and their approaches\n\
                 - Recent developments and news\n\
                 - Technical details and specifications",
                self.topic
            ))
            .expected_output(
                "A comprehensive research document containing:\n\
                 - Executive summary (2-3 paragraphs)\n\
                 - Key findings (5-10 bullet points)\n\
                 - Detailed analysis of each key finding\n\
                 - List of sources consulted"
            )
            .agent("topic-researcher")
            .timeout(300)
            .build();

        let analysis_task = Task::builder()
            .id("data-analysis")
            .name("Research Data Analysis")
            .description(
                "Analyze the research findings and extract:\n\
                 - Key statistics and metrics\n\
                 - Trend patterns\n\
                 - Comparison matrices\n\
                 - Actionable insights"
            )
            .expected_output(
                "An analysis report containing:\n\
                 - Summary of key statistics\n\
                 - Trend analysis with predictions\n\
                 - Comparison table of key options/technologies\n\
                 - Top 3 actionable recommendations"
            )
            .agent("data-analyst")
            .depends_on("initial-research")
            .timeout(180)
            .build();

        let synthesis_task = Task::builder()
            .id("research-synthesis")
            .name("Research Synthesis")
            .description(
                "Combine the research and analysis into a final research brief \
                 that can be used by writers to create content."
            )
            .expected_output(
                "A research brief containing:\n\
                 - Topic overview\n\
                 - Key facts and figures\n\
                 - Main talking points\n\
                 - Suggested content angles"
            )
            .agent("topic-researcher")
            .depends_on("data-analysis")
            .timeout(120)
            .build();

        // === CREW ===
        self.research_crew = Some(
            Crew::builder()
                .id("research-crew")
                .name("Research Team")
                .description("Team responsible for researching and analyzing the topic")
                .agent(topic_researcher)
                .agent(data_analyst)
                .task(research_task)
                .task(analysis_task)
                .task(synthesis_task)
                .process(Process::Sequential)
                .verbose(true)
                .build()
        );
    }

    /// Build the writing crew
    fn build_writing_crew(&mut self) {
        // === AGENTS ===
        let content_writer = Agent::builder()
            .id("content-writer")
            .role("Senior Content Writer")
            .goal("Create engaging, well-structured content based on research")
            .backstory(
                "You are an award-winning content writer with expertise in technology topics. \
                 You have a talent for making complex subjects accessible and engaging. \
                 Your writing is clear, concise, and captures reader attention while \
                 maintaining technical accuracy."
            )
            .model("gpt-4")
            .temperature(0.7)
            .with_short_term_memory()
            .build();

        let seo_specialist = Agent::builder()
            .id("seo-specialist")
            .role("SEO Specialist")
            .goal("Optimize content for search engines and readability")
            .backstory(
                "You are an SEO expert who understands how to structure content for \
                 maximum visibility and engagement. You know keyword optimization, \
                 meta descriptions, and content structure best practices."
            )
            .model("gpt-4")
            .temperature(0.4)
            .tool_name("keyword_analyzer")
            .build();

        // === TASKS ===
        let outline_task = Task::builder()
            .id("content-outline")
            .name("Create Content Outline")
            .description(
                "Based on the research brief, create a detailed content outline including:\n\
                 - Compelling title options\n\
                 - Section structure with headings\n\
                 - Key points for each section\n\
                 - Call-to-action ideas"
            )
            .expected_output(
                "A detailed outline with:\n\
                 - 3 title options\n\
                 - 5-7 main sections with sub-points\n\
                 - Suggested word count per section\n\
                 - Tone and style guidelines"
            )
            .agent("content-writer")
            .timeout(120)
            .build();

        let draft_task = Task::builder()
            .id("content-draft")
            .name("Write Content Draft")
            .description(
                "Write the full content draft following the outline. \
                 Include all sections, examples, and supporting details."
            )
            .expected_output(
                "A complete content draft of 1500-2000 words including:\n\
                 - Engaging introduction\n\
                 - Well-structured body sections\n\
                 - Practical examples or case studies\n\
                 - Strong conclusion with call-to-action"
            )
            .agent("content-writer")
            .depends_on("content-outline")
            .timeout(300)
            .build();

        let seo_task = Task::builder()
            .id("seo-optimization")
            .name("SEO Optimization")
            .description(
                "Optimize the content for SEO including:\n\
                 - Keyword placement and density\n\
                 - Meta title and description\n\
                 - Heading structure optimization\n\
                 - Internal/external link suggestions"
            )
            .expected_output(
                "SEO-optimized content with:\n\
                 - Optimized headings\n\
                 - Meta title (under 60 chars)\n\
                 - Meta description (under 160 chars)\n\
                 - Primary and secondary keyword placements\n\
                 - Suggested links to add"
            )
            .agent("seo-specialist")
            .depends_on("content-draft")
            .timeout(180)
            .build();

        // === CREW ===
        self.writing_crew = Some(
            Crew::builder()
                .id("writing-crew")
                .name("Writing Team")
                .description("Team responsible for creating and optimizing content")
                .agent(content_writer)
                .agent(seo_specialist)
                .task(outline_task)
                .task(draft_task)
                .task(seo_task)
                .process(Process::Sequential)
                .verbose(true)
                .build()
        );
    }

    /// Build the review crew
    fn build_review_crew(&mut self) {
        // === AGENTS ===
        let editor = Agent::builder()
            .id("editor")
            .role("Senior Editor")
            .goal("Ensure content quality, accuracy, and readability")
            .backstory(
                "You are a meticulous editor with 20 years of experience in publishing. \
                 You have an eye for detail and can spot issues in grammar, style, \
                 and structure. You provide constructive feedback to improve content."
            )
            .model("gpt-4")
            .temperature(0.2)
            .build();

        let fact_checker = Agent::builder()
            .id("fact-checker")
            .role("Fact Checker")
            .goal("Verify all claims and statistics in the content")
            .backstory(
                "You are a thorough fact-checker who verifies every claim. \
                 You check sources, validate statistics, and ensure accuracy."
            )
            .model("gpt-4")
            .temperature(0.1)
            .tool_name("fact_verification")
            .build();

        // === TASKS ===
        let fact_check_task = Task::builder()
            .id("fact-checking")
            .name("Fact Checking")
            .description(
                "Verify all facts, statistics, and claims in the content. \
                 Flag any unverifiable or questionable claims."
            )
            .expected_output(
                "Fact check report with:\n\
                 - List of verified facts\n\
                 - List of unverifiable claims\n\
                 - Suggested corrections\n\
                 - Overall accuracy score (1-10)"
            )
            .agent("fact-checker")
            .timeout(180)
            .build();

        let editing_task = Task::builder()
            .id("content-editing")
            .name("Content Editing")
            .description(
                "Review and edit the content for:\n\
                 - Grammar and spelling\n\
                 - Style consistency\n\
                 - Flow and readability\n\
                 - Overall structure"
            )
            .expected_output(
                "Edited content with:\n\
                 - All grammar/spelling corrections\n\
                 - Style improvements\n\
                 - Restructuring if needed\n\
                 - Editor's notes and suggestions"
            )
            .agent("editor")
            .depends_on("fact-checking")
            .timeout(240)
            .build();

        let approval_task = Task::builder()
            .id("final-approval")
            .name("Final Approval Decision")
            .description(
                "Make final approval decision based on:\n\
                 - Fact check results\n\
                 - Editing quality\n\
                 - Overall content quality\n\n\
                 Respond with 'approved' if ready to publish, \
                 or 'revision needed: [reasons]' if changes are required."
            )
            .expected_output(
                "Either:\n\
                 - 'approved: Content is ready for publication'\n\
                 - 'revision needed: [specific issues to fix]'"
            )
            .agent("editor")
            .depends_on("content-editing")
            .timeout(60)
            .build();

        // === CREW ===
        self.review_crew = Some(
            Crew::builder()
                .id("review-crew")
                .name("Review Team")
                .description("Team responsible for fact-checking and editing content")
                .agent(editor)
                .agent(fact_checker)
                .task(fact_check_task)
                .task(editing_task)
                .task(approval_task)
                .process(Process::Sequential)
                .verbose(true)
                .build()
        );
    }

    /// Build the revision crew
    fn build_revision_crew(&mut self) {
        // === AGENTS ===
        let reviser = Agent::builder()
            .id("reviser")
            .role("Content Reviser")
            .goal("Incorporate feedback and improve content quality")
            .backstory(
                "You are skilled at taking feedback and improving content. \
                 You can quickly identify issues and make effective corrections."
            )
            .model("gpt-4")
            .temperature(0.5)
            .build();

        // === TASKS ===
        let revision_task = Task::builder()
            .id("revise-content")
            .name("Revise Content")
            .description(
                "Based on the review feedback, revise the content to:\n\
                 - Fix identified issues\n\
                 - Improve problem areas\n\
                 - Enhance overall quality"
            )
            .expected_output(
                "Revised content that addresses all feedback points"
            )
            .agent("reviser")
            .timeout(300)
            .build();

        // === CREW ===
        self.revision_crew = Some(
            Crew::builder()
                .id("revision-crew")
                .name("Revision Team")
                .description("Team responsible for incorporating feedback")
                .agent(reviser)
                .task(revision_task)
                .process(Process::Sequential)
                .verbose(true)
                .build()
        );
    }

    /// Build the flow that orchestrates all crews
    pub fn build_flow(&self) -> Flow {
        // Define states for each stage of the pipeline
        let research_state = FlowState::new("research", "Research Phase")
            .initial()
            .with_description("Conduct research on the topic")
            .with_crew("research-crew");

        let writing_state = FlowState::new("writing", "Writing Phase")
            .with_description("Create content based on research")
            .with_crew("writing-crew");

        let review_state = FlowState::new("review", "Review Phase")
            .with_description("Review and approve content")
            .with_crew("review-crew");

        let revision_state = FlowState::new("revision", "Revision Phase")
            .with_description("Revise content based on feedback")
            .with_crew("revision-crew");

        let published_state = FlowState::new("published", "Published")
            .final_state()
            .with_description("Content is published and pipeline complete");

        // Define transitions
        let research_to_writing = StateTransition::new("research", "writing")
            .when(TransitionCondition::OnSuccess)
            .with_description("Move to writing after research completes");

        let writing_to_review = StateTransition::new("writing", "review")
            .when(TransitionCondition::OnSuccess)
            .with_description("Move to review after writing completes");

        let review_approved = StateTransition::new("review", "published")
            .when(TransitionCondition::OutputContains("approved".to_string()))
            .with_priority(10)
            .with_description("Publish if content is approved");

        let review_needs_revision = StateTransition::new("review", "revision")
            .when(TransitionCondition::OutputContains("revision".to_string()))
            .with_priority(5)
            .with_description("Revise if content needs changes");

        let revision_to_review = StateTransition::new("revision", "review")
            .when(TransitionCondition::OnSuccess)
            .with_description("Return to review after revision");

        // Build the flow
        Flow::builder()
            .id("content-pipeline-flow")
            .name("Content Creation Pipeline")
            .description("Complete content creation workflow from research to publication")
            .state(research_state)
            .state(writing_state)
            .state(review_state)
            .state(revision_state)
            .state(published_state)
            .transition(research_to_writing)
            .transition(writing_to_review)
            .transition(review_approved)
            .transition(review_needs_revision)
            .transition(revision_to_review)
            .max_iterations(5) // Allow up to 5 revision cycles
            .variable("topic", serde_json::json!(self.topic.clone()))
            .build()
    }

    /// Run the complete pipeline
    pub async fn run(&mut self) -> Result<PipelineResult, String> {
        let start_time = std::time::Instant::now();
        let mut crew_results = Vec::new();
        let mut stats = PipelineStats::default();

        // Store topic in shared memory
        self.shared_memory
            .store("topic", serde_json::json!(self.topic.clone()))
            .await
            .map_err(|e| e.to_string())?;

        // Execute research crew
        if let Some(ref mut crew) = self.research_crew {
            stats.total_agents += crew.agents().len();
            stats.total_tasks += crew.tasks().len();

            let result = crew.kickoff().await.map_err(|e| e.to_string())?;
            stats.tasks_completed += result.task_outputs.len();
            stats.crews_executed += 1;

            // Store research output in memory
            self.shared_memory
                .store("research_output", serde_json::json!(result.output.clone()))
                .await
                .map_err(|e| e.to_string())?;

            crew_results.push(result);
        }

        // Execute writing crew
        if let Some(ref mut crew) = self.writing_crew {
            stats.total_agents += crew.agents().len();
            stats.total_tasks += crew.tasks().len();

            let result = crew.kickoff().await.map_err(|e| e.to_string())?;
            stats.tasks_completed += result.task_outputs.len();
            stats.crews_executed += 1;

            // Store writing output in memory
            self.shared_memory
                .store("writing_output", serde_json::json!(result.output.clone()))
                .await
                .map_err(|e| e.to_string())?;

            crew_results.push(result);
        }

        // Execute review crew
        if let Some(ref mut crew) = self.review_crew {
            stats.total_agents += crew.agents().len();
            stats.total_tasks += crew.tasks().len();

            let result = crew.kickoff().await.map_err(|e| e.to_string())?;
            stats.tasks_completed += result.task_outputs.len();
            stats.crews_executed += 1;

            crew_results.push(result);
        }

        let total_time_ms = start_time.elapsed().as_millis() as u64;

        // Combine all outputs
        let final_output = crew_results
            .iter()
            .map(|r| r.output.clone())
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        Ok(PipelineResult {
            final_output,
            crew_results,
            total_time_ms,
            stats,
        })
    }

    /// Get the topic
    pub fn topic(&self) -> &str {
        &self.topic
    }

    /// Get shared memory for inspection
    pub fn shared_memory(&self) -> &Memory {
        &self.shared_memory
    }
}

// ============================================================================
// STANDALONE FUNCTIONS FOR QUICK USAGE
// ============================================================================

/// Run a complete content pipeline for a given topic
///
/// This is a convenience function that creates and runs the entire pipeline.
///
/// # Example
///
/// ```rust,ignore
/// let result = run_content_pipeline("Rust AI Frameworks").await?;
/// println!("Created content: {}", result.final_output);
/// ```
pub async fn run_content_pipeline(topic: &str) -> Result<PipelineResult, String> {
    let mut pipeline = ContentPipeline::new(topic);
    pipeline.run().await
}

// ============================================================================
// SIMPLER STANDALONE EXAMPLE
// ============================================================================

/// A simpler example showing just crew execution with task dependencies
///
/// This demonstrates the core workflow without the complexity of flows.
pub async fn run_simple_crew_example() -> Result<CrewResult, String> {
    // Create agents
    let researcher = Agent::builder()
        .id("researcher")
        .role("Researcher")
        .goal("Find and analyze information")
        .backstory("Expert researcher")
        .model("gpt-4")
        .temperature(0.3)
        .build();

    let writer = Agent::builder()
        .id("writer")
        .role("Writer")
        .goal("Create compelling content")
        .backstory("Expert content creator")
        .model("gpt-4")
        .temperature(0.7)
        .build();

    let editor = Agent::builder()
        .id("editor")
        .role("Editor")
        .goal("Polish and perfect content")
        .backstory("Experienced editor")
        .model("gpt-4")
        .temperature(0.2)
        .build();

    // Create tasks with dependencies
    // Task flow: research -> write -> edit
    let research_task = Task::builder()
        .id("research")
        .name("Research Task")
        .description("Research the topic thoroughly")
        .expected_output("Research findings with key points")
        .agent("researcher")
        .build();

    let write_task = Task::builder()
        .id("write")
        .name("Writing Task")
        .description("Write content based on research")
        .expected_output("Draft content")
        .agent("writer")
        .depends_on("research") // Depends on research
        .build();

    let edit_task = Task::builder()
        .id("edit")
        .name("Editing Task")
        .description("Edit and polish the content")
        .expected_output("Final polished content")
        .agent("editor")
        .depends_on("write") // Depends on writing
        .build();

    // Create and run crew
    let mut crew = Crew::builder()
        .id("simple-crew")
        .name("Simple Content Crew")
        .agent(researcher)
        .agent(writer)
        .agent(editor)
        .task(research_task)
        .task(write_task)
        .task(edit_task)
        .process(Process::Sequential)
        .verbose(true)
        .build();

    crew.kickoff().await.map_err(|e| e.to_string())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_pipeline_creation() {
        let pipeline = ContentPipeline::new("Test Topic");

        assert_eq!(pipeline.topic(), "Test Topic");
        assert!(pipeline.research_crew.is_some());
        assert!(pipeline.writing_crew.is_some());
        assert!(pipeline.review_crew.is_some());
        assert!(pipeline.revision_crew.is_some());
    }

    #[test]
    fn test_pipeline_flow_creation() {
        let pipeline = ContentPipeline::new("Test Topic");
        let flow = pipeline.build_flow();

        assert_eq!(flow.id(), "content-pipeline-flow");
        assert!(flow.validate().is_ok());
        assert!(flow.initial_state().is_some());
        assert_eq!(flow.initial_state().unwrap().id, "research");
    }

    #[tokio::test]
    async fn test_pipeline_run() {
        let mut pipeline = ContentPipeline::new("AI Agents");
        let result = pipeline.run().await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.final_output.is_empty());
        assert!(!result.crew_results.is_empty());
        assert!(result.stats.crews_executed >= 1);
    }

    #[tokio::test]
    async fn test_simple_crew_example() {
        let result = run_simple_crew_example().await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(!result.task_outputs.is_empty());
    }

    #[tokio::test]
    async fn test_shared_memory_in_pipeline() {
        let mut pipeline = ContentPipeline::new("Memory Test");
        let _ = pipeline.run().await;

        // Check that topic was stored in shared memory
        let topic = pipeline.shared_memory().retrieve("topic").await;
        assert!(topic.is_ok());
        assert_eq!(topic.unwrap(), Some(serde_json::json!("Memory Test")));
    }

    #[tokio::test]
    async fn test_task_dependency_flow() {
        // Test that tasks execute in dependency order
        let agent = Agent::builder()
            .id("worker")
            .role("Worker")
            .goal("Work")
            .backstory("Worker")
            .build();

        let task_a = Task::builder()
            .id("task-a")
            .description("Task A - no deps")
            .expected_output("Output A")
            .agent("worker")
            .build();

        let task_b = Task::builder()
            .id("task-b")
            .description("Task B - depends on A")
            .expected_output("Output B")
            .agent("worker")
            .depends_on("task-a")
            .build();

        let task_c = Task::builder()
            .id("task-c")
            .description("Task C - depends on B")
            .expected_output("Output C")
            .agent("worker")
            .depends_on("task-b")
            .build();

        let mut crew = Crew::builder()
            .agent(agent)
            .task(task_a)
            .task(task_b)
            .task(task_c)
            .process(Process::Sequential)
            .build();

        let result = crew.kickoff().await.unwrap();

        // All tasks should complete in order
        assert!(result.success);
        assert!(result.task_outputs.contains_key("task-a"));
        assert!(result.task_outputs.contains_key("task-b"));
        assert!(result.task_outputs.contains_key("task-c"));
    }
}
