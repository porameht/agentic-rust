//! CrewAI-style Prompt Configuration
//!
//! This module provides a comprehensive prompt configuration system
//! following CrewAI patterns with support for:
//! - Template variables ({role}, {goal}, {backstory}, {task}, etc.)
//! - Multi-language prompts
//! - TOML configuration file loading
//! - Default prompts for common agent types
//!
//! # Configuration Structure
//!
//! ```toml
//! [crew.prompts]
//! # Agent system prompts
//! [crew.prompts.agent]
//! system = "You are {role}..."
//! task_execution = "Execute the following task..."
//! tool_usage = "You have access to the following tools..."
//!
//! # Task prompts
//! [crew.prompts.task]
//! description = "# Task\n{description}..."
//! context = "# Context from previous tasks..."
//!
//! # Crew prompts
//! [crew.prompts.crew]
//! delegation = "Delegate this task to..."
//! collaboration = "Work together with..."
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

/// Global crew prompt configuration singleton
static CREW_PROMPT_CONFIG: OnceLock<CrewPromptConfig> = OnceLock::new();

/// Get or initialize the global crew prompt configuration
pub fn crew_prompts() -> &'static CrewPromptConfig {
    CREW_PROMPT_CONFIG.get_or_init(CrewPromptConfig::load)
}

// ============================================================================
// PROMPT TEMPLATES
// ============================================================================

/// Template for agent system prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPromptTemplates {
    /// Main system prompt template
    /// Variables: {role}, {goal}, {backstory}
    #[serde(default = "default_agent_system")]
    pub system: String,

    /// Task execution prompt
    /// Variables: {task_description}, {expected_output}, {context}
    #[serde(default = "default_task_execution")]
    pub task_execution: String,

    /// Tool usage prompt
    /// Variables: {tools}
    #[serde(default = "default_tool_usage")]
    pub tool_usage: String,

    /// Memory/context prompt
    /// Variables: {memory_items}
    #[serde(default = "default_memory_prompt")]
    pub memory: String,

    /// Delegation prompt (for hierarchical process)
    /// Variables: {delegated_to}, {task_description}
    #[serde(default = "default_delegation")]
    pub delegation: String,

    /// Collaboration prompt
    /// Variables: {collaborators}, {shared_goal}
    #[serde(default = "default_collaboration")]
    pub collaboration: String,

    /// Final answer prompt
    /// Variables: {task_description}
    #[serde(default = "default_final_answer")]
    pub final_answer: String,

    /// Error handling prompt
    #[serde(default = "default_error_handling")]
    pub error_handling: String,
}

impl Default for AgentPromptTemplates {
    fn default() -> Self {
        Self {
            system: default_agent_system(),
            task_execution: default_task_execution(),
            tool_usage: default_tool_usage(),
            memory: default_memory_prompt(),
            delegation: default_delegation(),
            collaboration: default_collaboration(),
            final_answer: default_final_answer(),
            error_handling: default_error_handling(),
        }
    }
}

/// Template for task prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPromptTemplates {
    /// Task description template
    /// Variables: {description}, {expected_output}
    #[serde(default = "default_task_description")]
    pub description: String,

    /// Context from dependencies
    /// Variables: {source_task}, {context_output}
    #[serde(default = "default_task_context")]
    pub context: String,

    /// Task result format
    #[serde(default = "default_task_result")]
    pub result_format: String,

    /// Human input request
    #[serde(default = "default_human_input")]
    pub human_input: String,
}

impl Default for TaskPromptTemplates {
    fn default() -> Self {
        Self {
            description: default_task_description(),
            context: default_task_context(),
            result_format: default_task_result(),
            human_input: default_human_input(),
        }
    }
}

/// Template for crew-level prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewPromptTemplates {
    /// Manager prompt for hierarchical process
    /// Variables: {agents}, {tasks}, {goal}
    #[serde(default = "default_manager_prompt")]
    pub manager: String,

    /// Crew objective prompt
    /// Variables: {crew_name}, {crew_description}
    #[serde(default = "default_crew_objective")]
    pub objective: String,

    /// Task assignment prompt
    /// Variables: {agent_role}, {task_description}
    #[serde(default = "default_task_assignment")]
    pub task_assignment: String,

    /// Crew completion prompt
    #[serde(default = "default_crew_completion")]
    pub completion: String,
}

impl Default for CrewPromptTemplates {
    fn default() -> Self {
        Self {
            manager: default_manager_prompt(),
            objective: default_crew_objective(),
            task_assignment: default_task_assignment(),
            completion: default_crew_completion(),
        }
    }
}

// ============================================================================
// ROLE-SPECIFIC PROMPTS
// ============================================================================

/// Pre-defined role prompts for common agent types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePrompts {
    /// Researcher role prompt
    #[serde(default = "default_researcher_role")]
    pub researcher: RolePromptConfig,

    /// Writer role prompt
    #[serde(default = "default_writer_role")]
    pub writer: RolePromptConfig,

    /// Analyst role prompt
    #[serde(default = "default_analyst_role")]
    pub analyst: RolePromptConfig,

    /// Editor role prompt
    #[serde(default = "default_editor_role")]
    pub editor: RolePromptConfig,

    /// Manager role prompt
    #[serde(default = "default_manager_role")]
    pub manager: RolePromptConfig,

    /// Developer role prompt
    #[serde(default = "default_developer_role")]
    pub developer: RolePromptConfig,

    /// Reviewer role prompt
    #[serde(default = "default_reviewer_role")]
    pub reviewer: RolePromptConfig,

    /// Custom roles
    #[serde(default)]
    pub custom: HashMap<String, RolePromptConfig>,
}

impl Default for RolePrompts {
    fn default() -> Self {
        Self {
            researcher: default_researcher_role(),
            writer: default_writer_role(),
            analyst: default_analyst_role(),
            editor: default_editor_role(),
            manager: default_manager_role(),
            developer: default_developer_role(),
            reviewer: default_reviewer_role(),
            custom: HashMap::new(),
        }
    }
}

/// Configuration for a specific role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePromptConfig {
    /// Role name
    pub name: String,

    /// Role description/backstory template
    pub backstory: String,

    /// Default goal template
    pub default_goal: String,

    /// Suggested tools for this role
    #[serde(default)]
    pub suggested_tools: Vec<String>,

    /// Recommended temperature
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Additional system prompt additions
    #[serde(default)]
    pub system_additions: Option<String>,
}

// ============================================================================
// MAIN CONFIGURATION
// ============================================================================

/// Complete crew prompt configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewPromptConfig {
    /// Agent prompt templates
    #[serde(default)]
    pub agent: AgentPromptTemplates,

    /// Task prompt templates
    #[serde(default)]
    pub task: TaskPromptTemplates,

    /// Crew prompt templates
    #[serde(default)]
    pub crew: CrewPromptTemplates,

    /// Role-specific prompts
    #[serde(default)]
    pub roles: RolePrompts,

    /// Language-specific prompt variations
    #[serde(default)]
    pub i18n: HashMap<String, I18nPrompts>,
}

impl Default for CrewPromptConfig {
    fn default() -> Self {
        let mut i18n = HashMap::new();
        i18n.insert("th".to_string(), thai_prompts());
        i18n.insert("en".to_string(), english_prompts());

        Self {
            agent: AgentPromptTemplates::default(),
            task: TaskPromptTemplates::default(),
            crew: CrewPromptTemplates::default(),
            roles: RolePrompts::default(),
            i18n,
        }
    }
}

/// Language-specific prompt variations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct I18nPrompts {
    /// Agent system prompt in this language
    #[serde(default)]
    pub agent_system: Option<String>,

    /// Task description prefix
    #[serde(default)]
    pub task_prefix: Option<String>,

    /// Common phrases
    #[serde(default)]
    pub phrases: HashMap<String, String>,
}

impl CrewPromptConfig {
    /// Load from TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read crew prompts file: {}", e))?;
        Self::from_toml(&content)
    }

    /// Parse from TOML string
    pub fn from_toml(content: &str) -> Result<Self, String> {
        toml::from_str(content).map_err(|e| format!("Failed to parse crew prompts: {}", e))
    }

    /// Load from default paths or use defaults
    pub fn load() -> Self {
        let paths = [
            "config/crew_prompts.toml",
            "config/prompts.toml",
            "./crew_prompts.toml",
        ];

        for path in paths {
            if Path::new(path).exists() {
                match Self::from_file(path) {
                    Ok(config) => {
                        tracing::info!("Loaded crew prompts from: {}", path);
                        return config;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load crew prompts from {}: {}", path, e);
                    }
                }
            }
        }

        tracing::info!("Using default crew prompts");
        Self::default()
    }

    /// Get role configuration by name
    pub fn get_role(&self, name: &str) -> Option<&RolePromptConfig> {
        match name.to_lowercase().as_str() {
            "researcher" => Some(&self.roles.researcher),
            "writer" => Some(&self.roles.writer),
            "analyst" => Some(&self.roles.analyst),
            "editor" => Some(&self.roles.editor),
            "manager" => Some(&self.roles.manager),
            "developer" => Some(&self.roles.developer),
            "reviewer" => Some(&self.roles.reviewer),
            _ => self.roles.custom.get(name),
        }
    }

    /// Get language-specific prompts
    pub fn get_i18n(&self, lang: &str) -> Option<&I18nPrompts> {
        self.i18n.get(lang)
    }

    /// Build agent system prompt with variable substitution
    pub fn build_agent_prompt(
        &self,
        role: &str,
        goal: &str,
        backstory: &str,
        tools: &[String],
        language: Option<&str>,
    ) -> String {
        let mut prompt = self.agent.system.clone();

        // Substitute variables
        prompt = prompt.replace("{role}", role);
        prompt = prompt.replace("{goal}", goal);
        prompt = prompt.replace("{backstory}", backstory);

        // Add tool usage section if tools are provided
        if !tools.is_empty() {
            let tools_str = tools.join(", ");
            let tool_section = self.agent.tool_usage.replace("{tools}", &tools_str);
            prompt.push_str("\n\n");
            prompt.push_str(&tool_section);
        }

        // Apply language-specific modifications
        if let Some(lang) = language {
            if let Some(i18n) = self.get_i18n(lang) {
                if let Some(agent_system) = &i18n.agent_system {
                    // Prepend or replace with language-specific system prompt
                    prompt = agent_system.replace("{role}", role);
                    prompt = prompt.replace("{goal}", goal);
                    prompt = prompt.replace("{backstory}", backstory);
                }
            }
        }

        prompt
    }

    /// Build task prompt with context
    pub fn build_task_prompt(
        &self,
        description: &str,
        expected_output: &str,
        context: &[(String, String)], // (source_task, output)
    ) -> String {
        let mut prompt = self.task.description.clone();

        prompt = prompt.replace("{description}", description);
        prompt = prompt.replace("{expected_output}", expected_output);

        // Add context from dependencies
        if !context.is_empty() {
            prompt.push_str("\n\n");
            for (source, output) in context {
                let ctx = self
                    .task
                    .context
                    .replace("{source_task}", source)
                    .replace("{context_output}", output);
                prompt.push_str(&ctx);
                prompt.push_str("\n");
            }
        }

        prompt
    }

    /// Build crew manager prompt for hierarchical process
    pub fn build_manager_prompt(
        &self,
        _crew_name: &str,
        agents: &[(&str, &str)], // (id, role)
        tasks: &[(&str, &str)],  // (id, description)
        goal: &str,
    ) -> String {
        let mut prompt = self.crew.manager.clone();

        let agents_str = agents
            .iter()
            .map(|(id, role)| format!("- {}: {}", id, role))
            .collect::<Vec<_>>()
            .join("\n");

        let tasks_str = tasks
            .iter()
            .map(|(id, desc)| format!("- {}: {}", id, desc))
            .collect::<Vec<_>>()
            .join("\n");

        prompt = prompt.replace("{agents}", &agents_str);
        prompt = prompt.replace("{tasks}", &tasks_str);
        prompt = prompt.replace("{goal}", goal);

        prompt
    }
}

// ============================================================================
// PROMPT BUILDER
// ============================================================================

/// Fluent builder for creating prompts
pub struct PromptBuilder {
    base: String,
    variables: HashMap<String, String>,
    sections: Vec<String>,
}

impl PromptBuilder {
    /// Create a new prompt builder with a base template
    pub fn new(template: &str) -> Self {
        Self {
            base: template.to_string(),
            variables: HashMap::new(),
            sections: Vec::new(),
        }
    }

    /// Create from agent config
    pub fn for_agent(config: &CrewPromptConfig) -> Self {
        Self::new(&config.agent.system)
    }

    /// Set a variable value
    pub fn var(mut self, name: &str, value: &str) -> Self {
        self.variables.insert(name.to_string(), value.to_string());
        self
    }

    /// Set role
    pub fn role(self, role: &str) -> Self {
        self.var("role", role)
    }

    /// Set goal
    pub fn goal(self, goal: &str) -> Self {
        self.var("goal", goal)
    }

    /// Set backstory
    pub fn backstory(self, backstory: &str) -> Self {
        self.var("backstory", backstory)
    }

    /// Set task description
    pub fn task(self, description: &str) -> Self {
        self.var("task_description", description)
    }

    /// Set expected output
    pub fn expected_output(self, output: &str) -> Self {
        self.var("expected_output", output)
    }

    /// Add a section to the prompt
    pub fn section(mut self, title: &str, content: &str) -> Self {
        self.sections.push(format!("## {}\n{}", title, content));
        self
    }

    /// Add tools section
    pub fn tools(self, tools: &[&str]) -> Self {
        if tools.is_empty() {
            return self;
        }
        let tools_list = tools
            .iter()
            .map(|t| format!("- `{}`", t))
            .collect::<Vec<_>>()
            .join("\n");
        self.section("Available Tools", &tools_list)
    }

    /// Add context section
    pub fn context(self, context: &str) -> Self {
        self.section("Context", context)
    }

    /// Add instructions section
    pub fn instructions(self, instructions: &str) -> Self {
        self.section("Instructions", instructions)
    }

    /// Build the final prompt
    pub fn build(self) -> String {
        let mut result = self.base.clone();

        // Replace all variables
        for (key, value) in &self.variables {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }

        // Append sections
        if !self.sections.is_empty() {
            result.push_str("\n\n");
            result.push_str(&self.sections.join("\n\n"));
        }

        result
    }
}

// ============================================================================
// DEFAULT PROMPT IMPLEMENTATIONS
// ============================================================================

fn default_agent_system() -> String {
    r#"You are {role}.

## Your Goal
{goal}

## Background
{backstory}

## Guidelines
- Focus on your assigned task and deliver high-quality results
- Use your expertise and tools effectively
- Communicate clearly and professionally
- If you need information you don't have, say so
- Provide detailed explanations when necessary"#
        .to_string()
}

fn default_task_execution() -> String {
    r#"# Task Execution

## Task Description
{task_description}

## Expected Output
{expected_output}

## Instructions
1. Read the task description carefully
2. Use available tools if needed
3. Apply your expertise to complete the task
4. Ensure your output matches the expected format
5. Review your work before submitting"#
        .to_string()
}

fn default_tool_usage() -> String {
    r#"## Available Tools
You have access to the following tools: {tools}

Use these tools when appropriate to complete your task. Call tools with proper arguments and handle their responses correctly."#.to_string()
}

fn default_memory_prompt() -> String {
    r#"## Relevant Context from Memory
{memory_items}

Use this context to inform your response, but prioritize the current task requirements."#
        .to_string()
}

fn default_delegation() -> String {
    r#"## Delegation
You are delegating the following task to {delegated_to}:
{task_description}

Provide clear instructions and context for successful completion."#
        .to_string()
}

fn default_collaboration() -> String {
    r#"## Collaboration
You are working with: {collaborators}
Shared Goal: {shared_goal}

Coordinate effectively and build upon each other's work."#
        .to_string()
}

fn default_final_answer() -> String {
    r#"## Final Answer
Based on completing the task "{task_description}", provide your final response.

Ensure it matches the expected output format and contains all required information."#
        .to_string()
}

fn default_error_handling() -> String {
    r#"## Error Recovery
If you encounter an error or cannot complete a step:
1. Describe what went wrong
2. Explain what you attempted
3. Suggest alternative approaches if possible
4. Request clarification if needed"#
        .to_string()
}

fn default_task_description() -> String {
    r#"# Task
{description}

# Expected Output
{expected_output}"#
        .to_string()
}

fn default_task_context() -> String {
    r#"## Context from "{source_task}"
{context_output}
---"#
        .to_string()
}

fn default_task_result() -> String {
    r#"# Task Result

Present your findings and deliverables in a clear, structured format.
Ensure all requirements from the expected output are addressed."#
        .to_string()
}

fn default_human_input() -> String {
    r#"## Human Review Required
This task requires human validation before proceeding.
Please review the output and provide feedback or approval."#
        .to_string()
}

fn default_manager_prompt() -> String {
    r#"You are the Manager of this crew, responsible for coordinating the team to achieve the goal.

## Goal
{goal}

## Your Team
{agents}

## Tasks to Complete
{tasks}

## Your Responsibilities
1. Assign tasks to the most suitable agents based on their roles
2. Monitor progress and provide guidance
3. Ensure quality of deliverables
4. Coordinate handoffs between agents
5. Make final decisions when needed

Delegate effectively and ensure the team works together efficiently."#
        .to_string()
}

fn default_crew_objective() -> String {
    r#"# Crew: {crew_name}

{crew_description}

Work together as a team to accomplish the shared objectives."#
        .to_string()
}

fn default_task_assignment() -> String {
    r#"## Task Assignment
Agent: {agent_role}
Task: {task_description}

Complete this task using your expertise and available tools."#
        .to_string()
}

fn default_crew_completion() -> String {
    r#"# Crew Execution Complete

All tasks have been processed. Review the combined outputs for the final deliverable."#
        .to_string()
}

fn default_temperature() -> f32 {
    0.7
}

// ============================================================================
// ROLE DEFAULTS
// ============================================================================

fn default_researcher_role() -> RolePromptConfig {
    RolePromptConfig {
        name: "Researcher".to_string(),
        backstory: "You are an experienced researcher with a talent for finding, analyzing, and synthesizing information from multiple sources. You have strong critical thinking skills and attention to detail.".to_string(),
        default_goal: "Conduct thorough research and provide accurate, well-sourced insights".to_string(),
        suggested_tools: vec!["web_search".to_string(), "document_reader".to_string()],
        temperature: 0.3,
        system_additions: Some("Always cite your sources and distinguish between facts and opinions.".to_string()),
    }
}

fn default_writer_role() -> RolePromptConfig {
    RolePromptConfig {
        name: "Writer".to_string(),
        backstory: "You are a skilled content writer who creates engaging, clear, and well-structured content. You adapt your writing style to the audience and purpose.".to_string(),
        default_goal: "Create compelling, high-quality written content".to_string(),
        suggested_tools: vec![],
        temperature: 0.7,
        system_additions: Some("Focus on clarity, engagement, and meeting the target audience's needs.".to_string()),
    }
}

fn default_analyst_role() -> RolePromptConfig {
    RolePromptConfig {
        name: "Analyst".to_string(),
        backstory: "You are a data analyst skilled at extracting insights from information, identifying patterns, and making data-driven recommendations.".to_string(),
        default_goal: "Analyze data and provide actionable insights".to_string(),
        suggested_tools: vec!["data_analysis".to_string()],
        temperature: 0.2,
        system_additions: Some("Support conclusions with data and present findings clearly.".to_string()),
    }
}

fn default_editor_role() -> RolePromptConfig {
    RolePromptConfig {
        name: "Editor".to_string(),
        backstory: "You are an experienced editor with an eye for detail. You improve content quality, fix errors, and ensure consistency and clarity.".to_string(),
        default_goal: "Polish and perfect content to publication quality".to_string(),
        suggested_tools: vec![],
        temperature: 0.2,
        system_additions: Some("Focus on grammar, style, clarity, and overall quality.".to_string()),
    }
}

fn default_manager_role() -> RolePromptConfig {
    RolePromptConfig {
        name: "Manager".to_string(),
        backstory: "You are an experienced project manager skilled at coordinating teams, delegating tasks, and ensuring successful project completion.".to_string(),
        default_goal: "Coordinate the team effectively to achieve project goals".to_string(),
        suggested_tools: vec![],
        temperature: 0.4,
        system_additions: Some("Delegate based on team strengths and monitor progress.".to_string()),
    }
}

fn default_developer_role() -> RolePromptConfig {
    RolePromptConfig {
        name: "Developer".to_string(),
        backstory: "You are a skilled software developer with expertise in writing clean, efficient, and maintainable code. You follow best practices and write comprehensive tests.".to_string(),
        default_goal: "Write high-quality code that meets requirements".to_string(),
        suggested_tools: vec!["code_search".to_string(), "file_editor".to_string()],
        temperature: 0.3,
        system_additions: Some("Write clean, tested, and well-documented code.".to_string()),
    }
}

fn default_reviewer_role() -> RolePromptConfig {
    RolePromptConfig {
        name: "Reviewer".to_string(),
        backstory: "You are a thorough reviewer who examines work critically, identifies issues, and provides constructive feedback.".to_string(),
        default_goal: "Review work and provide actionable feedback".to_string(),
        suggested_tools: vec![],
        temperature: 0.2,
        system_additions: Some("Be thorough but constructive in your feedback.".to_string()),
    }
}

// ============================================================================
// I18N DEFAULTS
// ============================================================================

fn thai_prompts() -> I18nPrompts {
    let mut phrases = HashMap::new();
    phrases.insert("task".to_string(), "งาน".to_string());
    phrases.insert("goal".to_string(), "เป้าหมาย".to_string());
    phrases.insert("output".to_string(), "ผลลัพธ์".to_string());
    phrases.insert("context".to_string(), "บริบท".to_string());
    phrases.insert("tools".to_string(), "เครื่องมือ".to_string());
    phrases.insert("instructions".to_string(), "คำแนะนำ".to_string());

    I18nPrompts {
        agent_system: Some(
            r#"คุณคือ {role}

## เป้าหมายของคุณ
{goal}

## ประวัติ
{backstory}

## แนวทางการทำงาน
- มุ่งเน้นที่งานที่ได้รับมอบหมายและส่งมอบผลงานคุณภาพสูง
- ใช้ความเชี่ยวชาญและเครื่องมือของคุณอย่างมีประสิทธิภาพ
- สื่อสารอย่างชัดเจนและเป็นมืออาชีพ
- หากต้องการข้อมูลที่ไม่มี ให้บอกตรงๆ
- ให้คำอธิบายโดยละเอียดเมื่อจำเป็น"#
                .to_string(),
        ),
        task_prefix: Some("# งาน\n".to_string()),
        phrases,
    }
}

fn english_prompts() -> I18nPrompts {
    let mut phrases = HashMap::new();
    phrases.insert("task".to_string(), "Task".to_string());
    phrases.insert("goal".to_string(), "Goal".to_string());
    phrases.insert("output".to_string(), "Output".to_string());
    phrases.insert("context".to_string(), "Context".to_string());
    phrases.insert("tools".to_string(), "Tools".to_string());
    phrases.insert("instructions".to_string(), "Instructions".to_string());

    I18nPrompts {
        agent_system: None, // Use default English
        task_prefix: None,
        phrases,
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CrewPromptConfig::default();

        assert!(!config.agent.system.is_empty());
        assert!(!config.task.description.is_empty());
        assert!(!config.crew.manager.is_empty());
    }

    #[test]
    fn test_build_agent_prompt() {
        let config = CrewPromptConfig::default();

        let prompt = config.build_agent_prompt(
            "Senior Researcher",
            "Find accurate information",
            "Expert with 10 years experience",
            &["web_search".to_string(), "document_reader".to_string()],
            None,
        );

        assert!(prompt.contains("Senior Researcher"));
        assert!(prompt.contains("Find accurate information"));
        assert!(prompt.contains("10 years experience"));
        assert!(prompt.contains("web_search"));
    }

    #[test]
    fn test_build_task_prompt() {
        let config = CrewPromptConfig::default();

        let context = vec![(
            "research".to_string(),
            "Research findings here".to_string(),
        )];

        let prompt = config.build_task_prompt(
            "Write a blog post",
            "1500 word article",
            &context,
        );

        assert!(prompt.contains("Write a blog post"));
        assert!(prompt.contains("1500 word article"));
        assert!(prompt.contains("Research findings here"));
    }

    #[test]
    fn test_prompt_builder() {
        let prompt = PromptBuilder::new("You are {role} with goal: {goal}")
            .role("Researcher")
            .goal("Find information")
            .section("Context", "Some context here")
            .tools(&["search", "read"])
            .build();

        assert!(prompt.contains("You are Researcher"));
        assert!(prompt.contains("Find information"));
        assert!(prompt.contains("Some context here"));
        assert!(prompt.contains("search"));
    }

    #[test]
    fn test_get_role() {
        let config = CrewPromptConfig::default();

        let researcher = config.get_role("researcher");
        assert!(researcher.is_some());
        assert_eq!(researcher.unwrap().name, "Researcher");

        let writer = config.get_role("Writer"); // Test case insensitivity
        assert!(writer.is_some());
    }

    #[test]
    fn test_i18n_prompts() {
        let config = CrewPromptConfig::default();

        let thai = config.get_i18n("th");
        assert!(thai.is_some());
        assert!(thai.unwrap().agent_system.is_some());

        let english = config.get_i18n("en");
        assert!(english.is_some());
    }

    #[test]
    fn test_build_manager_prompt() {
        let config = CrewPromptConfig::default();

        let agents = vec![
            ("researcher", "Senior Researcher"),
            ("writer", "Content Writer"),
        ];
        let tasks = vec![
            ("research", "Research the topic"),
            ("write", "Write the article"),
        ];

        let prompt = config.build_manager_prompt(
            "Content Crew",
            &agents,
            &tasks,
            "Create high-quality content",
        );

        assert!(prompt.contains("Senior Researcher"));
        assert!(prompt.contains("Content Writer"));
        assert!(prompt.contains("Research the topic"));
        assert!(prompt.contains("Create high-quality content"));
    }

    #[test]
    fn test_toml_parsing() {
        let toml_str = r#"
[agent]
system = "Custom agent prompt for {role}"

[task]
description = "Custom task: {description}"

[roles.custom.my_role]
name = "Custom Role"
backstory = "Custom backstory"
default_goal = "Custom goal"
temperature = 0.5
"#;

        let config = CrewPromptConfig::from_toml(toml_str).unwrap();
        assert!(config.agent.system.contains("Custom agent prompt"));
        assert!(config.roles.custom.contains_key("my_role"));
    }
}
