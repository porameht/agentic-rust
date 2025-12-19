//! Example Crew Configurations
//!
//! This module provides pre-configured crews for common use cases.

use super::agent::Agent;
use super::crew::Crew;
use super::flow::{Flow, FlowState, StateTransition, TransitionCondition};
use super::memory::{MemoryConfig, MemoryType};
use super::process::Process;
use super::task::Task;

/// Creates a research crew with a researcher and writer
///
/// # Example
/// ```rust,ignore
/// use agent::crew::examples::create_research_crew;
///
/// let mut crew = create_research_crew("AI Agents", "blog post");
/// let result = crew.kickoff().await?;
/// ```
pub fn create_research_crew(topic: &str, output_type: &str) -> Crew {
    // Create the researcher agent
    let researcher = Agent::builder()
        .id("researcher")
        .role("Senior Research Analyst")
        .goal(format!("Conduct comprehensive research on {}", topic))
        .backstory(
            "You are a seasoned research analyst with over 15 years of experience in \
             technology research. You have a keen eye for identifying trends, analyzing \
             complex topics, and synthesizing information from multiple sources. Your \
             research has been featured in top tech publications.",
        )
        .model("gpt-4")
        .temperature(0.3)
        .verbose(true)
        .with_long_term_memory()
        .build();

    // Create the writer agent
    let writer = Agent::builder()
        .id("writer")
        .role("Content Writer")
        .goal(format!(
            "Create a compelling {} based on research findings",
            output_type
        ))
        .backstory(
            "You are an award-winning content writer with expertise in technology topics. \
             You have a talent for making complex subjects accessible and engaging. Your \
             writing style is clear, concise, and captures reader attention while \
             maintaining technical accuracy.",
        )
        .model("gpt-4")
        .temperature(0.7)
        .verbose(true)
        .build();

    // Create research task
    let research_task = Task::builder()
        .id("research-task")
        .name("Research Task")
        .description(format!(
            "Conduct thorough research on {}. \
             Identify key concepts, trends, challenges, and opportunities. \
             Gather data from multiple perspectives and sources.",
            topic
        ))
        .expected_output(
            "A comprehensive research report with:\n\
             - Executive summary\n\
             - Key findings (at least 5)\n\
             - Supporting evidence and data\n\
             - Industry trends\n\
             - Recommendations",
        )
        .agent("researcher")
        .build();

    // Create writing task
    let writing_task = Task::builder()
        .id("writing-task")
        .name("Writing Task")
        .description(format!(
            "Based on the research findings, create a compelling {}. \
             Make it engaging, informative, and actionable for the target audience.",
            output_type
        ))
        .expected_output(format!(
            "A polished {} that:\n\
             - Has a captivating introduction\n\
             - Presents key insights clearly\n\
             - Uses examples and analogies\n\
             - Has a strong conclusion with call-to-action\n\
             - Is approximately 1500-2000 words",
            output_type
        ))
        .agent("writer")
        .depends_on("research-task")
        .build();

    // Build the crew
    Crew::builder()
        .id("research-crew")
        .name("Research & Writing Crew")
        .description(format!("A crew specialized in researching {} and creating {}", topic, output_type))
        .agent(researcher)
        .agent(writer)
        .task(research_task)
        .task(writing_task)
        .process(Process::Sequential)
        .verbose(true)
        .build()
}

/// Creates a sales support crew with product expert and customer success agents
///
/// # Example
/// ```rust,ignore
/// use agent::crew::examples::create_sales_crew;
///
/// let mut crew = create_sales_crew("Enterprise Software", "Thai");
/// let result = crew.kickoff().await?;
/// ```
pub fn create_sales_crew(product_category: &str, language: &str) -> Crew {
    // Product Expert Agent
    let product_expert = Agent::builder()
        .id("product-expert")
        .role("Product Expert")
        .goal(format!(
            "Provide accurate and detailed information about {} products",
            product_category
        ))
        .backstory(format!(
            "You are a product specialist with deep knowledge of {}. \
             You understand technical specifications, features, and use cases. \
             You can explain complex features in simple terms and match \
             products to customer needs. You speak {}.",
            product_category, language
        ))
        .model("gpt-4")
        .temperature(0.3)
        .tool_name("product_search")
        .tool_name("get_brochure")
        .with_short_term_memory()
        .build();

    // Customer Success Agent
    let customer_success = Agent::builder()
        .id("customer-success")
        .role("Customer Success Manager")
        .goal("Ensure customer satisfaction and drive successful outcomes")
        .backstory(format!(
            "You are an experienced customer success manager who excels at \
             understanding customer needs and providing solutions. You have \
             excellent communication skills in {} and can handle objections \
             gracefully while building long-term relationships.",
            language
        ))
        .model("gpt-4")
        .temperature(0.5)
        .tool_name("company_info")
        .build();

    // Sales Coordinator Agent
    let sales_coordinator = Agent::builder()
        .id("sales-coordinator")
        .role("Sales Coordinator")
        .goal("Coordinate the sales process and ensure smooth customer journey")
        .backstory(
            "You are a detail-oriented sales coordinator who ensures all customer \
             interactions are properly tracked and followed up. You coordinate \
             between product experts and customer success to provide a seamless \
             experience.",
        )
        .model("gpt-4")
        .temperature(0.4)
        .allow_delegation(true)
        .build();

    // Initial qualification task
    let qualification_task = Task::builder()
        .id("qualify-lead")
        .name("Lead Qualification")
        .description(
            "Analyze the customer inquiry and qualify the lead. \
             Understand their needs, budget, timeline, and decision-making process.",
        )
        .expected_output(
            "Lead qualification summary:\n\
             - Customer needs analysis\n\
             - Budget indication\n\
             - Timeline expectations\n\
             - Decision makers involved\n\
             - Qualification score (1-10)",
        )
        .agent("sales-coordinator")
        .build();

    // Product recommendation task
    let recommendation_task = Task::builder()
        .id("recommend-products")
        .name("Product Recommendation")
        .description(
            "Based on the qualification, recommend suitable products. \
             Provide detailed explanations of how each product addresses \
             the customer's specific needs.",
        )
        .expected_output(
            "Product recommendations:\n\
             - Top 3 recommended products with justification\n\
             - Feature comparison table\n\
             - Pricing summary\n\
             - Available brochures and resources",
        )
        .agent("product-expert")
        .depends_on("qualify-lead")
        .build();

    // Follow-up task
    let followup_task = Task::builder()
        .id("prepare-followup")
        .name("Prepare Follow-up")
        .description(
            "Prepare a personalized follow-up plan for the customer. \
             Include next steps, answers to potential objections, and \
             additional resources.",
        )
        .expected_output(
            "Follow-up plan:\n\
             - Personalized email draft\n\
             - Answers to likely objections\n\
             - Next meeting agenda\n\
             - Additional resources to share",
        )
        .agent("customer-success")
        .depends_on("recommend-products")
        .build();

    Crew::builder()
        .id("sales-crew")
        .name("Sales Support Crew")
        .description(format!("Sales crew for {} in {}", product_category, language))
        .agent(product_expert)
        .agent(customer_success)
        .agent(sales_coordinator)
        .task(qualification_task)
        .task(recommendation_task)
        .task(followup_task)
        .process(Process::Sequential)
        .verbose(true)
        .memory(MemoryConfig {
            memory_type: MemoryType::ShortTerm,
            max_items: 100,
            ..Default::default()
        })
        .build()
}

/// Creates a code review crew with multiple specialized reviewers
///
/// # Example
/// ```rust,ignore
/// use agent::crew::examples::create_code_review_crew;
///
/// let mut crew = create_code_review_crew("Rust");
/// let result = crew.kickoff().await?;
/// ```
pub fn create_code_review_crew(language: &str) -> Crew {
    // Security Reviewer
    let security_reviewer = Agent::builder()
        .id("security-reviewer")
        .role("Security Engineer")
        .goal("Identify security vulnerabilities and recommend fixes")
        .backstory(format!(
            "You are a security engineer specializing in {} applications. \
             You have deep knowledge of OWASP vulnerabilities, secure coding \
             practices, and can identify potential attack vectors. You've \
             helped secure applications for Fortune 500 companies.",
            language
        ))
        .model("gpt-4")
        .temperature(0.2)
        .build();

    // Performance Reviewer
    let performance_reviewer = Agent::builder()
        .id("performance-reviewer")
        .role("Performance Engineer")
        .goal("Identify performance bottlenecks and optimization opportunities")
        .backstory(format!(
            "You are a performance optimization expert for {} applications. \
             You understand memory management, CPU optimization, async patterns, \
             and can spot inefficient algorithms. You've optimized systems \
             handling millions of requests per second.",
            language
        ))
        .model("gpt-4")
        .temperature(0.2)
        .build();

    // Code Quality Reviewer
    let quality_reviewer = Agent::builder()
        .id("quality-reviewer")
        .role("Senior Code Reviewer")
        .goal("Ensure code quality, maintainability, and best practices")
        .backstory(format!(
            "You are a senior developer with 20+ years of experience in {}. \
             You have a keen eye for code smells, design pattern violations, \
             and maintainability issues. You believe in clean, readable code \
             that other developers can easily understand and extend.",
            language
        ))
        .model("gpt-4")
        .temperature(0.3)
        .build();

    // Security review task
    let security_task = Task::builder()
        .id("security-review")
        .name("Security Review")
        .description(
            "Review the code for security vulnerabilities including:\n\
             - Input validation\n\
             - Authentication/authorization issues\n\
             - Injection vulnerabilities\n\
             - Sensitive data exposure\n\
             - Security misconfiguration",
        )
        .expected_output(
            "Security review report:\n\
             - Critical vulnerabilities found\n\
             - High/Medium/Low severity issues\n\
             - Remediation recommendations\n\
             - Security best practices to apply",
        )
        .agent("security-reviewer")
        .build();

    // Performance review task
    let performance_task = Task::builder()
        .id("performance-review")
        .name("Performance Review")
        .description(
            "Review the code for performance issues including:\n\
             - Algorithm complexity\n\
             - Memory allocations\n\
             - I/O bottlenecks\n\
             - Caching opportunities\n\
             - Async/parallel processing",
        )
        .expected_output(
            "Performance review report:\n\
             - Performance bottlenecks identified\n\
             - Optimization recommendations\n\
             - Estimated impact of each fix\n\
             - Benchmarking suggestions",
        )
        .agent("performance-reviewer")
        .build();

    // Quality review task
    let quality_task = Task::builder()
        .id("quality-review")
        .name("Code Quality Review")
        .description(
            "Review the code for quality and maintainability:\n\
             - Code organization\n\
             - Design patterns\n\
             - Error handling\n\
             - Documentation\n\
             - Test coverage",
        )
        .expected_output(
            "Code quality report:\n\
             - Code smells and anti-patterns\n\
             - Refactoring suggestions\n\
             - Documentation improvements\n\
             - Test coverage gaps",
        )
        .agent("quality-reviewer")
        .build();

    // Summary task
    let summary_task = Task::builder()
        .id("review-summary")
        .name("Review Summary")
        .description("Compile all reviews into a comprehensive summary with prioritized action items.")
        .expected_output(
            "Comprehensive review summary:\n\
             - Executive summary\n\
             - Priority 1 (Critical) items\n\
             - Priority 2 (Important) items\n\
             - Priority 3 (Nice to have) items\n\
             - Overall code quality score (1-10)",
        )
        .agent("quality-reviewer")
        .depends_on("security-review")
        .depends_on("performance-review")
        .depends_on("quality-review")
        .build();

    Crew::builder()
        .id("code-review-crew")
        .name("Code Review Crew")
        .description(format!("Comprehensive code review crew for {}", language))
        .agent(security_reviewer)
        .agent(performance_reviewer)
        .agent(quality_reviewer)
        .task(security_task)
        .task(performance_task)
        .task(quality_task)
        .task(summary_task)
        .process(Process::Sequential) // In production, first 3 tasks could be parallel
        .verbose(true)
        .build()
}

/// Creates a content creation flow with research, writing, editing, and publishing states
///
/// # Example
/// ```rust,ignore
/// use agent::crew::examples::create_content_flow;
///
/// let mut flow = create_content_flow();
/// let result = flow.run().await?;
/// ```
pub fn create_content_flow() -> Flow {
    // Define states
    let research_state = FlowState::new("research", "Research Phase")
        .initial()
        .with_description("Conduct research on the topic")
        .with_crew("research-crew");

    let writing_state = FlowState::new("writing", "Writing Phase")
        .with_description("Write the initial draft")
        .with_crew("writing-crew");

    let editing_state = FlowState::new("editing", "Editing Phase")
        .with_description("Edit and refine the content")
        .with_crew("editing-crew");

    let review_state = FlowState::new("review", "Review Phase")
        .with_description("Final review before publishing");

    let published_state = FlowState::new("published", "Published")
        .final_state()
        .with_description("Content has been published");

    let revision_state = FlowState::new("revision", "Revision Needed")
        .with_description("Content needs revisions")
        .with_crew("revision-crew");

    // Define transitions
    let research_to_writing = StateTransition::new("research", "writing")
        .when(TransitionCondition::OnSuccess)
        .with_description("Move to writing after successful research");

    let writing_to_editing = StateTransition::new("writing", "editing")
        .when(TransitionCondition::OnSuccess);

    let editing_to_review = StateTransition::new("editing", "review")
        .when(TransitionCondition::OnSuccess);

    let review_approved = StateTransition::new("review", "published")
        .when(TransitionCondition::OutputContains("approved".to_string()))
        .with_priority(10);

    let review_rejected = StateTransition::new("review", "revision")
        .when(TransitionCondition::OutputContains("revision".to_string()))
        .with_priority(5);

    let revision_to_editing = StateTransition::new("revision", "editing")
        .when(TransitionCondition::OnSuccess);

    // Build the flow
    Flow::builder()
        .id("content-flow")
        .name("Content Creation Flow")
        .description("End-to-end content creation workflow")
        .state(research_state)
        .state(writing_state)
        .state(editing_state)
        .state(review_state)
        .state(published_state)
        .state(revision_state)
        .transition(research_to_writing)
        .transition(writing_to_editing)
        .transition(editing_to_review)
        .transition(review_approved)
        .transition(review_rejected)
        .transition(revision_to_editing)
        .max_iterations(10)
        .build()
}

/// Creates a customer support flow with triage, handling, and escalation paths
pub fn create_support_flow() -> Flow {
    // States
    let triage = FlowState::new("triage", "Ticket Triage")
        .initial()
        .with_description("Analyze and categorize the support ticket")
        .with_crew("triage-crew");

    let simple_resolution = FlowState::new("simple", "Simple Resolution")
        .with_description("Handle simple issues with automated responses")
        .with_crew("simple-resolution-crew");

    let complex_handling = FlowState::new("complex", "Complex Issue Handling")
        .with_description("Handle complex issues requiring investigation")
        .with_crew("investigation-crew");

    let escalation = FlowState::new("escalation", "Escalation")
        .with_description("Escalate to human support")
        .with_crew("escalation-crew");

    let resolved = FlowState::new("resolved", "Resolved")
        .final_state()
        .with_description("Issue has been resolved");

    let escalated = FlowState::new("escalated", "Escalated to Human")
        .final_state()
        .with_description("Issue escalated to human support");

    // Transitions
    let triage_simple = StateTransition::new("triage", "simple")
        .when(TransitionCondition::OutputContains("simple".to_string()))
        .with_priority(10);

    let triage_complex = StateTransition::new("triage", "complex")
        .when(TransitionCondition::OutputContains("complex".to_string()))
        .with_priority(5);

    let triage_escalate = StateTransition::new("triage", "escalation")
        .when(TransitionCondition::OutputContains("escalate".to_string()))
        .with_priority(1);

    let simple_resolved = StateTransition::new("simple", "resolved")
        .when(TransitionCondition::OnSuccess);

    let complex_resolved = StateTransition::new("complex", "resolved")
        .when(TransitionCondition::OnSuccess);

    let complex_escalate = StateTransition::new("complex", "escalation")
        .when(TransitionCondition::OnFailure);

    let escalation_done = StateTransition::new("escalation", "escalated")
        .when(TransitionCondition::Always);

    Flow::builder()
        .id("support-flow")
        .name("Customer Support Flow")
        .description("Automated customer support workflow with escalation")
        .state(triage)
        .state(simple_resolution)
        .state(complex_handling)
        .state(escalation)
        .state(resolved)
        .state(escalated)
        .transition(triage_simple)
        .transition(triage_complex)
        .transition(triage_escalate)
        .transition(simple_resolved)
        .transition(complex_resolved)
        .transition(complex_escalate)
        .transition(escalation_done)
        .max_iterations(20)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_research_crew_creation() {
        let crew = create_research_crew("AI Agents", "blog post");
        assert_eq!(crew.id(), "research-crew");
        assert_eq!(crew.agents().len(), 2);
        assert_eq!(crew.tasks().len(), 2);
    }

    #[test]
    fn test_sales_crew_creation() {
        let crew = create_sales_crew("Enterprise Software", "English");
        assert_eq!(crew.id(), "sales-crew");
        assert_eq!(crew.agents().len(), 3);
        assert_eq!(crew.tasks().len(), 3);
    }

    #[test]
    fn test_code_review_crew_creation() {
        let crew = create_code_review_crew("Rust");
        assert_eq!(crew.id(), "code-review-crew");
        assert_eq!(crew.agents().len(), 3);
        assert_eq!(crew.tasks().len(), 4);
    }

    #[test]
    fn test_content_flow_creation() {
        let flow = create_content_flow();
        assert_eq!(flow.id(), "content-flow");
        assert!(flow.validate().is_ok());
    }

    #[test]
    fn test_support_flow_creation() {
        let flow = create_support_flow();
        assert_eq!(flow.id(), "support-flow");
        assert!(flow.validate().is_ok());
    }
}
