//! Output formatting and callbacks for Ralph execution.
//!
//! This module provides debug-level-aware output through ADK callbacks.
//! The callbacks are designed to be added to agents for clean, configurable logging.
//!
//! ## Debug Levels
//!
//! - `Minimal`: Only errors and final status
//! - `Normal`: Human-readable progress (default) - shows task progress, phase changes
//! - `Verbose`: Detailed output with tool calls and responses
//! - `Debug`: Full debug output with all internal state

use crate::models::DebugLevel;
use adk_rust::Part;
use colored::Colorize;

/// Output handler that respects debug levels.
///
/// This struct provides methods to create ADK callbacks that output
/// information based on the configured debug level.
#[derive(Debug, Clone)]
pub struct RalphOutput {
    level: DebugLevel,
}

impl Default for RalphOutput {
    fn default() -> Self {
        Self::new(DebugLevel::Normal)
    }
}

impl RalphOutput {
    /// Create a new output handler with the specified debug level.
    pub fn new(level: DebugLevel) -> Self {
        Self { level }
    }

    /// Get the current debug level.
    pub fn level(&self) -> DebugLevel {
        self.level
    }

    // =========================================================================
    // Direct output methods (for use outside callbacks)
    // =========================================================================

    /// Print a phase header (shown at Normal and above).
    pub fn phase(&self, name: &str) {
        if self.level.is_normal() {
            println!("\n{} {}", "▶".bright_cyan(), name.bright_white().bold());
        }
    }

    /// Print a status message within a phase (shown at Normal and above).
    pub fn status(&self, message: &str) {
        if self.level.is_normal() {
            println!("  {} {}", "•".bright_black(), message);
        }
    }

    /// Print a phase completion message (shown at Normal and above).
    pub fn phase_complete(&self, message: &str) {
        if self.level.is_normal() {
            println!("  {} {}", "✓".bright_green(), message.green());
        }
    }

    /// Print a list item (shown at Normal and above).
    pub fn list_item(&self, message: &str) {
        if self.level.is_normal() {
            println!("    {} {}", "─".bright_black(), message);
        }
    }

    /// Print a task start message (shown at Normal and above).
    pub fn task_start(&self, task_id: &str, title: &str) {
        if self.level.is_normal() {
            println!("  {} {} - {}", "→".bright_blue(), task_id.cyan(), title);
        }
    }

    /// Print a task completion message (shown at Normal and above).
    pub fn task_complete(&self, task_id: &str, success: bool) {
        if self.level.is_normal() {
            if success {
                println!("  {} {} completed", "✓".bright_green(), task_id.green());
            } else {
                println!("  {} {} failed", "✗".bright_red(), task_id.red());
            }
        }
    }

    /// Print iteration progress (shown at Normal and above).
    pub fn iteration(&self, current: u32, max: usize) {
        if self.level.is_normal() {
            println!(
                "  {} iteration {}/{}",
                "○".bright_black(),
                current,
                max
            );
        }
    }

    /// Print a progress bar for task completion (shown at Normal and above).
    ///
    /// Displays: `[████████░░░░░░░░░░░░] 40% (4/10 tasks)`
    pub fn progress_bar(&self, completed: usize, total: usize) {
        if !self.level.is_normal() || total == 0 {
            return;
        }

        let percentage = (completed * 100) / total;
        let bar_width = 30;
        let filled = (completed * bar_width) / total;
        let empty = bar_width - filled;

        let bar = format!(
            "{}{}",
            "█".repeat(filled).bright_green(),
            "░".repeat(empty).bright_black()
        );

        // Use carriage return to update in place
        print!(
            "\r  [{}] {}% ({}/{} tasks)  ",
            bar,
            percentage,
            completed,
            total
        );
        
        // Flush to ensure it displays
        use std::io::Write;
        let _ = std::io::stdout().flush();

        // Print newline when complete
        if completed == total {
            println!();
        }
    }

    /// Print a progress bar with a task name (shown at Normal and above).
    pub fn progress_bar_with_task(&self, completed: usize, total: usize, current_task: &str) {
        if !self.level.is_normal() || total == 0 {
            return;
        }

        let percentage = (completed * 100) / total;
        let bar_width = 20;
        let filled = (completed * bar_width) / total;
        let empty = bar_width - filled;

        let bar = format!(
            "{}{}",
            "█".repeat(filled).bright_green(),
            "░".repeat(empty).bright_black()
        );

        // Truncate task name if too long
        let task_display = if current_task.len() > 30 {
            format!("{}...", &current_task[..27])
        } else {
            current_task.to_string()
        };

        // Use carriage return to update in place
        print!(
            "\r  [{}] {}% │ {}  ",
            bar,
            percentage,
            task_display.cyan()
        );
        
        // Flush to ensure it displays
        use std::io::Write;
        let _ = std::io::stdout().flush();

        // Print newline when complete
        if completed == total {
            println!();
        }
    }

    /// Clear the current line (for progress bar updates).
    pub fn clear_line(&self) {
        if self.level.is_normal() {
            print!("\r{}\r", " ".repeat(80));
            use std::io::Write;
            let _ = std::io::stdout().flush();
        }
    }

    /// Print a tool call (shown at Verbose and above).
    pub fn tool_call(&self, name: &str, args: &serde_json::Value) {
        if self.level.is_verbose() {
            println!(
                "\n  {} {}",
                "🔧".bright_blue(),
                name.bright_white().bold()
            );
            if let Ok(pretty) = serde_json::to_string_pretty(args) {
                for line in pretty.lines() {
                    println!("     {}", line.bright_black());
                }
            }
        }
    }

    /// Print a tool response (shown at Verbose and above).
    pub fn tool_response(&self, name: &str, response: &serde_json::Value) {
        if self.level.is_verbose() {
            let resp_str = serde_json::to_string(response).unwrap_or_default();
            let display = if resp_str.len() > 300 {
                format!("{}...", &resp_str[..300])
            } else {
                resp_str
            };
            println!("     {} {}", "←".green(), display.bright_black());
        } else if self.level.is_debug() {
            println!("     {} {} response:", "←".green(), name.green());
            if let Ok(pretty) = serde_json::to_string_pretty(response) {
                for line in pretty.lines() {
                    println!("       {}", line.bright_black());
                }
            }
        }
    }

    /// Print a concise tool result summary at Normal level.
    ///
    /// Shows meaningful outcomes for key tools (tests, git, npm install)
    /// without the full JSON dump that Verbose shows.
    pub fn tool_result_summary(&self, name: &str, response: &serde_json::Value) {
        if !self.level.is_normal() || self.level.is_verbose() {
            return; // Verbose already shows full response via tool_response()
        }

        match name {
            "test" => {
                if let Some(results) = response.get("results") {
                    let passed = results.get("passed").and_then(|v| v.as_u64()).unwrap_or(0);
                    let failed = results.get("failed").and_then(|v| v.as_u64()).unwrap_or(0);
                    let skipped = results.get("skipped").and_then(|v| v.as_u64()).unwrap_or(0);
                    let all_passed = results.get("all_passed").and_then(|v| v.as_bool()).unwrap_or(false);

                    if all_passed {
                        println!(
                            "    {} Tests passed: {} passed{}",
                            "✓".bright_green(),
                            passed.to_string().green(),
                            if skipped > 0 { format!(", {} skipped", skipped) } else { String::new() }
                        );
                    } else {
                        println!(
                            "    {} Tests failed: {} passed, {} failed{}",
                            "✗".bright_red(),
                            passed,
                            failed.to_string().red(),
                            if skipped > 0 { format!(", {} skipped", skipped) } else { String::new() }
                        );
                        // Show a snippet of stderr if tests failed
                        if let Some(stderr) = response.get("stderr").and_then(|v| v.as_str()) {
                            let error_lines: Vec<&str> = stderr
                                .lines()
                                .filter(|l| {
                                    let lower = l.to_lowercase();
                                    lower.contains("fail") || lower.contains("error") || lower.contains("assert")
                                })
                                .take(3)
                                .collect();
                            for line in error_lines {
                                let trimmed = if line.len() > 100 { &line[..100] } else { line };
                                println!("      {} {}", "│".bright_red(), trimmed.bright_black());
                            }
                        }
                    }
                } else if let Some(msg) = response.get("message").and_then(|v| v.as_str()) {
                    // Fallback: detect/check operations
                    println!("    {} {}", "ℹ".bright_blue(), msg.bright_black());
                }
            }
            "git" => {
                if let Some(op) = response.get("operation").and_then(|v| v.as_str()) {
                    match op {
                        "commit" => {
                            if let Some(hash) = response.get("commit_hash").and_then(|v| v.as_str()) {
                                let msg = response.get("message").and_then(|v| v.as_str()).unwrap_or("");
                                let short_msg = if msg.len() > 50 { &msg[..50] } else { msg };
                                println!(
                                    "    {} Committed {} \"{}\"",
                                    "✓".bright_green(),
                                    hash[..7.min(hash.len())].bright_black(),
                                    short_msg
                                );
                            } else {
                                println!("    {} Committed", "✓".bright_green());
                            }
                        }
                        "add" => {
                            if let Some(files) = response.get("files").and_then(|v| v.as_array()) {
                                println!(
                                    "    {} Staged {} file(s)",
                                    "✓".bright_green(),
                                    files.len()
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
            "file" => {
                // File writes already show "Writing <path>" from FunctionCall handler.
                // Only show result for errors.
                if let Some(false) = response.get("success").and_then(|v| v.as_bool()) {
                    if let Some(op) = response.get("operation").and_then(|v| v.as_str()) {
                        let path = response.get("path").and_then(|v| v.as_str()).unwrap_or("unknown");
                        println!(
                            "    {} Failed to {} {}",
                            "✗".bright_red(),
                            op,
                            path.bright_black()
                        );
                    }
                }
            }
            "run_project" => {
                let success = response.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                if let Some(cmd) = response.get("command").and_then(|v| v.as_str()) {
                    if success {
                        println!("    {} `{}` succeeded", "✓".bright_green(), cmd.bright_black());
                    } else {
                        println!("    {} `{}` failed", "✗".bright_red(), cmd.bright_black());
                        if let Some(stderr) = response.get("stderr").and_then(|v| v.as_str()) {
                            let last_lines: Vec<&str> = stderr.lines().rev().take(2).collect();
                            for line in last_lines.iter().rev() {
                                let trimmed = if line.len() > 100 { &line[..100] } else { line };
                                println!("      {} {}", "│".bright_red(), trimmed.bright_black());
                            }
                        }
                    }
                }
            }
            "tasks" => {
                // Show task info from get_next responses
                if let Some(task) = response.get("task") {
                    let id = task.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                    let title = task.get("title").and_then(|v| v.as_str()).unwrap_or("");
                    println!(
                        "    {} Next: {} - {}",
                        "→".bright_blue(),
                        id.cyan(),
                        title
                    );
                } else if let Some(true) = response.get("all_complete").and_then(|v| v.as_bool()) {
                    println!("    {} All tasks complete", "✓".bright_green());
                } else if let Some(blocked) = response.get("blocked_count").and_then(|v| v.as_u64()) {
                    println!(
                        "    {} {} task(s) blocked",
                        "⚠".bright_yellow(),
                        blocked
                    );
                }
            }
            _ => {}
        }
    }

    /// Print LLM text output (shown at Verbose and above).
    pub fn llm_text(&self, text: &str) {
        if self.level.is_verbose() && !text.trim().is_empty() {
            println!("\n  {} {}", "💭".bright_magenta(), text.trim());
        }
    }

    /// Print debug information (shown at Debug only).
    pub fn debug(&self, context: &str, message: &str) {
        if self.level.is_debug() {
            println!(
                "  {} [{}] {}",
                "🐛".bright_yellow(),
                context.bright_black(),
                message
            );
        }
    }

    /// Print an error (always shown).
    pub fn error(&self, message: &str) {
        eprintln!("{} {}", "✗ Error:".bright_red().bold(), message);
    }

    /// Print a warning (shown at Normal and above).
    pub fn warn(&self, message: &str) {
        if self.level.is_normal() {
            println!("{} {}", "⚠".bright_yellow(), message.yellow());
        }
    }

    /// Print success message (always shown).
    pub fn success(&self, message: &str) {
        println!("{} {}", "✓".bright_green(), message.green());
    }

    /// Print the startup banner (shown at Normal and above).
    pub fn banner(&self) {
        if self.level.is_normal() {
            println!(
                "{}",
                r#"
  ____       _       _     
 |  _ \ __ _| |_ __ | |__  
 | |_) / _` | | '_ \| '_ \ 
 |  _ < (_| | | |_) | | | |
 |_| \_\__,_|_| .__/|_| |_|
              |_|          
"#
                .cyan()
            );
            println!(
                "{}",
                "Multi-Agent Autonomous Development System".bright_white()
            );
            println!();
        }
    }

    /// Print final summary (always shown except minimal only shows status).
    pub fn summary(&self, iterations: u32, tasks_completed: usize, tasks_total: usize, success: bool) {
        if self.level.is_minimal() {
            // Minimal: just the result
            if success {
                println!("✓ Complete: {}/{} tasks", tasks_completed, tasks_total);
            } else {
                println!("✗ Incomplete: {}/{} tasks in {} iterations", tasks_completed, tasks_total, iterations);
            }
        } else {
            // Normal and above: formatted summary
            println!();
            println!("{}", "─".repeat(50).bright_black());
            if success {
                println!(
                    "{} {} tasks completed in {} iterations",
                    "✓".bright_green(),
                    tasks_completed.to_string().green(),
                    iterations
                );
            } else {
                println!(
                    "{} {}/{} tasks completed in {} iterations",
                    "⚠".bright_yellow(),
                    tasks_completed,
                    tasks_total,
                    iterations
                );
            }
            println!("{}", "─".repeat(50).bright_black());
        }
    }
}

/// Process an event stream part and output based on debug level.
///
/// This is a helper for processing events from the agent run loop.
pub fn process_event_part(output: &RalphOutput, part: &Part) {
    match part {
        Part::FunctionCall { name, args, .. } => {
            output.tool_call(name, args);
        }
        Part::FunctionResponse { function_response, .. } => {
            // At Normal level, show concise result summaries for key tools
            output.tool_result_summary(&function_response.name, &function_response.response);
            // At Verbose+, show the full response
            output.tool_response(&function_response.name, &function_response.response);
        }
        Part::Text { text } => {
            output.llm_text(text);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_level_checks() {
        let minimal = RalphOutput::new(DebugLevel::Minimal);
        assert!(minimal.level().is_minimal());
        assert!(!minimal.level().is_normal());
        assert!(!minimal.level().is_verbose());
        assert!(!minimal.level().is_debug());

        let normal = RalphOutput::new(DebugLevel::Normal);
        assert!(!normal.level().is_minimal());
        assert!(normal.level().is_normal());
        assert!(!normal.level().is_verbose());
        assert!(!normal.level().is_debug());

        let verbose = RalphOutput::new(DebugLevel::Verbose);
        assert!(!verbose.level().is_minimal());
        assert!(verbose.level().is_normal());
        assert!(verbose.level().is_verbose());
        assert!(!verbose.level().is_debug());

        let debug = RalphOutput::new(DebugLevel::Debug);
        assert!(!debug.level().is_minimal());
        assert!(debug.level().is_normal());
        assert!(debug.level().is_verbose());
        assert!(debug.level().is_debug());
    }

    #[test]
    fn test_output_default() {
        let output = RalphOutput::default();
        assert_eq!(output.level(), DebugLevel::Normal);
    }
}
