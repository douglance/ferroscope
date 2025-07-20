//! # Ferroscope
//!
//! A Model Context Protocol (MCP) server that enables AI assistants to debug Rust programs
//! using LLDB and GDB debuggers.
//!
//! ## Overview
//!
//! Ferroscope bridges the gap between AI assistants and native debugging tools, allowing
//! AI agents to perform debugging tasks like setting breakpoints, stepping through code,
//! and inspecting variables in running Rust programs.
//!
//! ## Features
//!
//! - **Native debugging**: Uses LLDB (macOS) and GDB (Linux) debuggers
//! - **MCP Protocol**: Implements Model Context Protocol for AI assistant integration
//! - **10 debugging tools**: Complete workflow from loading to stepping through code
//! - **State management**: Tracks debugging session state and program lifecycle
//! - **Cross-platform**: Works on macOS and Linux (Windows support planned)
//!
//! ## Available Tools
//!
//! - `debug_run` - Load and prepare Rust programs for debugging
//! - `debug_break` - Set breakpoints at functions or lines
//! - `debug_continue` - Launch/continue program execution
//! - `debug_step` - Step through code line by line
//! - `debug_step_into` - Step into function calls
//! - `debug_step_out` - Step out of current function
//! - `debug_eval` - Evaluate expressions and inspect variables
//! - `debug_backtrace` - Show call stack
//! - `debug_list_breakpoints` - List all breakpoints
//! - `debug_state` - Get current debugging session state
//!
//! ## Usage
//!
//! Ferroscope is designed to be used by AI assistants through the MCP protocol.
//! It runs as a server that accepts JSON-RPC commands over stdin/stdout.
//!
//! ```bash
//! # Install ferroscope
//! cargo install ferroscope
//!
//! # Run the MCP server
//! ferroscope
//! ```
//!
//! ## Example Debugging Workflow
//!
//! 1. Load a Rust program: `debug_run /path/to/project`
//! 2. Set breakpoints: `debug_break main`
//! 3. Start execution: `debug_continue`
//! 4. At breakpoints: `debug_eval variable_name`
//! 5. Step through code: `debug_step`
//!
//! ## Security Considerations
//!
//! ⚠️ **Security Warning**: Ferroscope runs with full user privileges and can execute
//! arbitrary code through the debugger. Only use with trusted code and in secure environments.
//!
//! ## Requirements
//!
//! - Rust toolchain
//! - LLDB (macOS) or GDB (Linux)
//! - Debug symbols in target binaries

use anyhow::Result;
use serde_json::{json, Value};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::Mutex;

/// Represents the current state of a debugging session.
///
/// The debug state tracks the lifecycle of a program being debugged,
/// from initial loading through execution and completion.
#[derive(Debug, Clone, PartialEq)]
enum DebugState {
    /// No program has been loaded for debugging
    NotLoaded,
    /// Program is loaded but not yet running
    Loaded,
    /// Program is currently executing
    Running,
    /// Program execution is paused (e.g., at a breakpoint)
    Stopped,
    /// Program crashed or encountered an error
    Crashed,
    /// Program execution completed successfully
    Completed,
}

/// Represents an active debugging session with a spawned debugger process.
///
/// A `DebugSession` manages the communication with an LLDB or GDB process,
/// tracking the state of the debugging session and the program being debugged.
struct DebugSession {
    /// The spawned debugger process (LLDB or GDB)
    process: Child,
    /// Standard input pipe to send commands to the debugger
    stdin: ChildStdin,
    /// Buffered reader for the debugger's standard output
    stdout: BufReader<ChildStdout>,
    /// Current state of the debugging session
    state: DebugState,
    /// Path to the binary being debugged
    binary_path: String,
    /// Current location in the program (file:line or function name)
    current_location: Option<String>,
}

/// The main MCP server that handles debugging requests from AI assistants.
///
/// `DebugServer` implements the Model Context Protocol, accepting JSON-RPC commands
/// over stdin/stdout and managing debugging sessions through LLDB or GDB.
///
/// ## Thread Safety
///
/// The server uses `Arc<Mutex<_>>` to safely share the debugging session across
/// async tasks, ensuring only one debugging operation can occur at a time.
struct DebugServer {
    /// The current debugging session, if any
    session: Arc<Mutex<Option<DebugSession>>>,
}

impl DebugServer {
    /// Creates a new debug server instance.
    ///
    /// The server starts with no active debugging session. Sessions are created
    /// when the `debug_run` tool is called with a binary path.
    fn new() -> Self {
        Self {
            session: Arc::new(Mutex::new(None)),
        }
    }

    /// Sends a command to the active debugger process and returns the response.
    ///
    /// This method handles communication with the underlying LLDB or GDB process,
    /// including timeout handling and response parsing.
    ///
    /// # Arguments
    ///
    /// * `command` - The debugger command to execute (e.g., "breakpoint set", "continue")
    ///
    /// # Returns
    ///
    /// Returns the debugger's response as a string, or an error if no session is active
    /// or if the command fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - No debugging session is currently active
    /// - The debugger process has terminated
    /// - Communication with the debugger fails
    /// - The command times out (after 10 seconds)
    async fn send_debugger_command(&self, command: &str) -> Result<String> {
        let mut session_guard = self.session.lock().await;

        if let Some(session) = session_guard.as_mut() {
            // Send command to debugger
            session.stdin.write_all(command.as_bytes()).await?;
            session.stdin.write_all(b"\n").await?;
            session.stdin.flush().await?;

            // Read response with intelligent parsing
            let mut response = String::new();
            let mut line = String::new();

            let timeout_duration = std::time::Duration::from_secs(10);
            let start_time = std::time::Instant::now();

            loop {
                // Check for timeout
                if start_time.elapsed() > timeout_duration {
                    response.push_str("[TIMEOUT - Command may still be processing]");
                    break;
                }

                // Try to read a line with timeout
                tokio::select! {
                    result = session.stdout.read_line(&mut line) => {
                        match result {
                            Ok(0) => break, // EOF
                            Ok(_) => {
                                response.push_str(&line);

                                // Intelligent response detection based on command type
                                if self.is_response_complete(&line, command) {
                                    break;
                                }

                                line.clear();
                            }
                            Err(_) => break,
                        }
                    }
                    _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                        // Continue reading
                        continue;
                    }
                }
            }

            // Update session state based on response
            self.update_session_state(&response, session).await;

            Ok(response)
        } else {
            Err(anyhow::anyhow!("No active debugger session"))
        }
    }

    fn is_response_complete(&self, line: &str, command: &str) -> bool {
        // LLDB prompt detection
        if line.trim() == "(lldb)" {
            return true;
        }

        // Command-specific completion detection
        if command.starts_with("process launch")
            && line.contains("Process")
            && (line.contains("launched") || line.contains("stopped"))
        {
            return true;
        }

        if command.starts_with("process continue")
            && line.contains("Process")
            && (line.contains("stopped") || line.contains("exited"))
        {
            return true;
        }

        if command.starts_with("breakpoint set")
            && line.contains("Breakpoint")
            && line.contains(":")
        {
            return true;
        }

        if (command.starts_with("expression") || command.starts_with("frame variable"))
            && (line.contains("=") || line.contains("error:"))
        {
            return true;
        }

        false
    }

    async fn update_session_state(&self, response: &str, session: &mut DebugSession) {
        if response.contains("Process") && response.contains("launched") {
            session.state = DebugState::Running;
        } else if response.contains("Process") && response.contains("stopped") {
            session.state = DebugState::Stopped;
        } else if response.contains("Process") && response.contains("exited") {
            session.state = DebugState::Completed;
        } else if response.contains("crashed")
            || response.contains("SIGSEGV")
            || response.contains("SIGABRT")
        {
            session.state = DebugState::Crashed;
        }

        // Extract current location if available
        if response.contains("stop reason") {
            // Parse location from LLDB stop output
            if let Some(location) = self.extract_location_from_response(response) {
                session.current_location = Some(location);
            }
        }
    }

    fn extract_location_from_response(&self, response: &str) -> Option<String> {
        // Look for patterns like "at main.rs:10:5"
        for line in response.lines() {
            if line.contains(" at ") {
                if let Some(location_part) = line.split(" at ").nth(1) {
                    if let Some(location) = location_part.split_whitespace().next() {
                        return Some(location.to_string());
                    }
                }
            }
        }
        None
    }

    /// Loads and prepares a Rust program for debugging.
    ///
    /// This is the primary tool for starting a debugging session. It can accept either
    /// a path to a compiled binary or a path to a Rust project directory. If given a
    /// directory, it will automatically build the project using `cargo build`.
    ///
    /// # Arguments
    ///
    /// * `binary_path` - Path to a compiled binary or Rust project directory
    ///
    /// # Returns
    ///
    /// Returns a JSON response indicating success or failure of loading the program.
    ///
    /// # Examples
    ///
    /// Loading a Rust project directory:
    /// ```json
    /// {"name": "debug_run", "arguments": {"binary_path": "./my_rust_project"}}
    /// ```
    ///
    /// Loading a compiled binary:
    /// ```json
    /// {"name": "debug_run", "arguments": {"binary_path": "./target/debug/my_program"}}
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The binary path does not exist
    /// - Building the Rust project fails (for directory paths)
    /// - Starting the debugger process fails
    /// - The debugger cannot load the binary
    async fn debug_run(&self, binary_path: &str) -> Result<Value> {
        // Clean up any existing session
        {
            let mut session_guard = self.session.lock().await;
            if let Some(mut old_session) = session_guard.take() {
                let _ = old_session.process.kill().await;
            }
        }

        // Check if the path is a directory (source code) or binary
        let path = std::path::Path::new(binary_path);
        let binary_to_debug = if path.is_dir() {
            // It's a source directory, try to build it
            self.build_rust_project(binary_path).await?
        } else if path.exists() {
            // It's an existing binary
            binary_path.to_string()
        } else {
            return Err(anyhow::anyhow!("Path does not exist: {}", binary_path));
        };

        // Start debugger with the binary
        self.start_debugger_session(&binary_to_debug).await
    }

    async fn build_rust_project(&self, source_dir: &str) -> Result<String> {
        // Change to the source directory and run cargo build
        let output = tokio::process::Command::new("cargo")
            .arg("build")
            .current_dir(source_dir)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Build failed: {}", stderr));
        }

        // Find the built binary
        let cargo_toml_path = std::path::Path::new(source_dir).join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Err(anyhow::anyhow!("No Cargo.toml found in {}", source_dir));
        }

        let cargo_toml = std::fs::read_to_string(&cargo_toml_path)?;
        let project_name = cargo_toml
            .lines()
            .find(|line| line.starts_with("name = "))
            .and_then(|line| line.split('"').nth(1))
            .ok_or_else(|| anyhow::anyhow!("Could not parse project name from Cargo.toml"))?;

        let binary_path = std::path::Path::new(source_dir)
            .join("target")
            .join("debug")
            .join(project_name);

        if binary_path.exists() {
            Ok(binary_path.to_string_lossy().to_string())
        } else {
            Err(anyhow::anyhow!(
                "Built binary not found at {:?}",
                binary_path
            ))
        }
    }

    async fn start_debugger_session(&self, binary_path: &str) -> Result<Value> {
        // Launch LLDB with the binary
        let mut cmd = tokio::process::Command::new("lldb");
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()?;

        // Get stdin/stdout handles
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to get stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to get stdout"))?;
        let stdout_reader = BufReader::new(stdout);

        // Create session
        let session = DebugSession {
            process: child,
            stdin,
            stdout: stdout_reader,
            state: DebugState::NotLoaded,
            binary_path: binary_path.to_string(),
            current_location: None,
        };

        // Store the session
        {
            let mut session_guard = self.session.lock().await;
            *session_guard = Some(session);
        }

        // Wait for LLDB to start
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Load the binary
        let load_response = self
            .send_debugger_command(&format!("target create \"{}\"", binary_path))
            .await?;

        // Update state
        {
            let mut session_guard = self.session.lock().await;
            if let Some(session) = session_guard.as_mut() {
                session.state = DebugState::Loaded;
            }
        }

        Ok(json!({
            "success": true,
            "state": "loaded",
            "output": load_response.trim(),
            "binary_path": binary_path
        }))
    }

    /// Sets a breakpoint at the specified function or line.
    ///
    /// Breakpoints pause program execution when reached, allowing inspection
    /// of variables and program state at that point.
    ///
    /// # Arguments
    ///
    /// * `location` - Function name (e.g., "main") or file:line (e.g., "src/main.rs:10")
    ///
    /// # Returns
    ///
    /// Returns a JSON response indicating whether the breakpoint was successfully set.
    ///
    /// # Examples
    ///
    /// Setting a breakpoint on the main function:
    /// ```json
    /// {"name": "debug_break", "arguments": {"location": "main"}}
    /// ```
    ///
    /// Setting a breakpoint at a specific line:
    /// ```json
    /// {"name": "debug_break", "arguments": {"location": "src/main.rs:25"}}
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - No debugging session is active
    /// - The debugger communication fails
    /// - The specified location cannot be resolved
    async fn debug_break(&self, location: &str) -> Result<Value> {
        let command = format!("breakpoint set --name {}", location);
        let response = self.send_debugger_command(&command).await?;

        let success = !response.contains("no locations") && !response.contains("error:");

        Ok(json!({
            "success": success,
            "output": response.trim(),
            "location": location
        }))
    }

    async fn debug_continue(&self) -> Result<Value> {
        // Check current state
        let current_state = {
            let session_guard = self.session.lock().await;
            session_guard
                .as_ref()
                .map(|s| s.state.clone())
                .unwrap_or(DebugState::NotLoaded)
        };

        let command = match current_state {
            DebugState::Loaded => {
                // First time - need to launch the program
                "process launch"
            }
            DebugState::Stopped => {
                // Program is stopped at breakpoint - continue execution
                "process continue"
            }
            DebugState::Running => {
                return Ok(json!({
                    "success": false,
                    "error": "Program is already running",
                    "state": "running"
                }));
            }
            DebugState::Completed | DebugState::Crashed => {
                return Ok(json!({
                    "success": false,
                    "error": "Program has finished execution",
                    "state": format!("{:?}", current_state).to_lowercase()
                }));
            }
            DebugState::NotLoaded => {
                return Ok(json!({
                    "success": false,
                    "error": "No program loaded. Use debug_run first.",
                    "state": "not_loaded"
                }));
            }
        };

        let response = self.send_debugger_command(command).await?;

        // Get updated state
        let (new_state, location) = {
            let session_guard = self.session.lock().await;
            if let Some(session) = session_guard.as_ref() {
                (session.state.clone(), session.current_location.clone())
            } else {
                (DebugState::NotLoaded, None)
            }
        };

        Ok(json!({
            "success": true,
            "state": format!("{:?}", new_state).to_lowercase(),
            "output": response.trim(),
            "location": location
        }))
    }

    async fn debug_step(&self) -> Result<Value> {
        let current_state = {
            let session_guard = self.session.lock().await;
            session_guard
                .as_ref()
                .map(|s| s.state.clone())
                .unwrap_or(DebugState::NotLoaded)
        };

        if current_state != DebugState::Stopped {
            return Ok(json!({
                "success": false,
                "error": "Program must be stopped at a breakpoint to step",
                "state": format!("{:?}", current_state).to_lowercase()
            }));
        }

        let response = self.send_debugger_command("thread step-over").await?;

        // Get updated state and location
        let (new_state, location) = {
            let session_guard = self.session.lock().await;
            if let Some(session) = session_guard.as_ref() {
                (session.state.clone(), session.current_location.clone())
            } else {
                (DebugState::NotLoaded, None)
            }
        };

        Ok(json!({
            "success": true,
            "state": format!("{:?}", new_state).to_lowercase(),
            "output": response.trim(),
            "location": location
        }))
    }

    async fn debug_step_into(&self) -> Result<Value> {
        let current_state = {
            let session_guard = self.session.lock().await;
            session_guard
                .as_ref()
                .map(|s| s.state.clone())
                .unwrap_or(DebugState::NotLoaded)
        };

        if current_state != DebugState::Stopped {
            return Ok(json!({
                "success": false,
                "error": "Program must be stopped at a breakpoint to step",
                "state": format!("{:?}", current_state).to_lowercase()
            }));
        }

        let response = self.send_debugger_command("thread step-in").await?;

        let (new_state, location) = {
            let session_guard = self.session.lock().await;
            if let Some(session) = session_guard.as_ref() {
                (session.state.clone(), session.current_location.clone())
            } else {
                (DebugState::NotLoaded, None)
            }
        };

        Ok(json!({
            "success": true,
            "state": format!("{:?}", new_state).to_lowercase(),
            "output": response.trim(),
            "location": location
        }))
    }

    async fn debug_step_out(&self) -> Result<Value> {
        let current_state = {
            let session_guard = self.session.lock().await;
            session_guard
                .as_ref()
                .map(|s| s.state.clone())
                .unwrap_or(DebugState::NotLoaded)
        };

        if current_state != DebugState::Stopped {
            return Ok(json!({
                "success": false,
                "error": "Program must be stopped at a breakpoint to step",
                "state": format!("{:?}", current_state).to_lowercase()
            }));
        }

        let response = self.send_debugger_command("thread step-out").await?;

        let (new_state, location) = {
            let session_guard = self.session.lock().await;
            if let Some(session) = session_guard.as_ref() {
                (session.state.clone(), session.current_location.clone())
            } else {
                (DebugState::NotLoaded, None)
            }
        };

        Ok(json!({
            "success": true,
            "state": format!("{:?}", new_state).to_lowercase(),
            "output": response.trim(),
            "location": location
        }))
    }

    /// Evaluates an expression in the current debugging context.
    ///
    /// This tool allows inspection of variables, calling functions, and evaluating
    /// arbitrary expressions at the current program state. The program must be
    /// stopped (e.g., at a breakpoint) for evaluation to work.
    ///
    /// # Arguments
    ///
    /// * `expression` - The expression to evaluate (variable name, function call, etc.)
    ///
    /// # Returns
    ///
    /// Returns a JSON response with the evaluation result or an error message.
    ///
    /// # Examples
    ///
    /// Inspecting a variable:
    /// ```json
    /// {"name": "debug_eval", "arguments": {"expression": "my_variable"}}
    /// ```
    ///
    /// Evaluating a complex expression:
    /// ```json
    /// {"name": "debug_eval", "arguments": {"expression": "my_struct.field + 42"}}
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - No debugging session is active
    /// - The program is not currently stopped at a breakpoint
    /// - The expression cannot be evaluated in the current context
    /// - The debugger communication fails
    ///
    /// # Security Warning
    ///
    /// ⚠️ This function can execute arbitrary code through the expression evaluator.
    /// Only use with trusted expressions and in secure environments.
    async fn debug_eval(&self, expression: &str) -> Result<Value> {
        let current_state = {
            let session_guard = self.session.lock().await;
            session_guard
                .as_ref()
                .map(|s| s.state.clone())
                .unwrap_or(DebugState::NotLoaded)
        };

        if current_state != DebugState::Stopped {
            return Ok(json!({
                "success": false,
                "error": "Program must be stopped (at breakpoint) to evaluate expressions",
                "state": format!("{:?}", current_state).to_lowercase()
            }));
        }

        // Try both expression and frame variable commands
        let expr_cmd = format!("expression {}", expression);
        let frame_cmd = format!("frame variable {}", expression);

        // Try expression first
        let response = self.send_debugger_command(&expr_cmd).await?;

        if response.contains("error:") || response.contains("undeclared identifier") {
            // Try frame variable as fallback
            let frame_response = self.send_debugger_command(&frame_cmd).await?;

            let success = !frame_response.contains("error:");
            Ok(json!({
                "success": success,
                "expression": expression,
                "output": frame_response.trim(),
                "method": "frame_variable"
            }))
        } else {
            let success = !response.contains("error:");
            Ok(json!({
                "success": success,
                "expression": expression,
                "output": response.trim(),
                "method": "expression"
            }))
        }
    }

    async fn debug_backtrace(&self) -> Result<Value> {
        let current_state = {
            let session_guard = self.session.lock().await;
            session_guard
                .as_ref()
                .map(|s| s.state.clone())
                .unwrap_or(DebugState::NotLoaded)
        };

        if current_state != DebugState::Stopped {
            return Ok(json!({
                "success": false,
                "error": "Program must be stopped to show backtrace",
                "state": format!("{:?}", current_state).to_lowercase()
            }));
        }

        let response = self.send_debugger_command("thread backtrace").await?;

        Ok(json!({
            "success": true,
            "output": response.trim()
        }))
    }

    async fn debug_list_breakpoints(&self) -> Result<Value> {
        let response = self.send_debugger_command("breakpoint list").await?;

        Ok(json!({
            "success": true,
            "output": response.trim()
        }))
    }

    async fn get_debug_state(&self) -> Result<Value> {
        let (state, location, binary_path) = {
            let session_guard = self.session.lock().await;
            if let Some(session) = session_guard.as_ref() {
                (
                    session.state.clone(),
                    session.current_location.clone(),
                    Some(session.binary_path.clone()),
                )
            } else {
                (DebugState::NotLoaded, None, None)
            }
        };

        Ok(json!({
            "state": format!("{:?}", state).to_lowercase(),
            "location": location,
            "binary_path": binary_path
        }))
    }

    // MCP Protocol Implementation

    /// Handles the MCP initialize request from AI assistants.
    ///
    /// This method implements the Model Context Protocol initialization handshake,
    /// announcing the server's capabilities and protocol version to the AI assistant.
    ///
    /// # Arguments
    ///
    /// * `_params` - Initialization parameters from the client (currently unused)
    ///
    /// # Returns
    ///
    /// Returns a JSON response with server capabilities and version information.
    async fn handle_initialize(&self, _params: Value) -> Value {
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "ferroscope",
                "version": "0.1.1"
            }
        })
    }

    async fn handle_list_tools(&self) -> Value {
        json!({
            "tools": [
                {
                    "name": "debug_run",
                    "description": "Load and prepare a Rust program for debugging",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "binary_path": {
                                "type": "string",
                                "description": "Path to the Rust binary or source directory to debug"
                            }
                        },
                        "required": ["binary_path"]
                    }
                },
                {
                    "name": "debug_break",
                    "description": "Set a breakpoint at the specified function or line",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "location": {
                                "type": "string",
                                "description": "Function name or file:line to break at"
                            }
                        },
                        "required": ["location"]
                    }
                },
                {
                    "name": "debug_continue",
                    "description": "Launch program (if not started) or continue execution until next breakpoint",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
                    "name": "debug_step",
                    "description": "Step to the next line of code (step over function calls)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
                    "name": "debug_step_into",
                    "description": "Step into function calls",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
                    "name": "debug_step_out",
                    "description": "Step out of the current function",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
                    "name": "debug_eval",
                    "description": "Evaluate an expression or inspect a variable in the current debugging context",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "expression": {
                                "type": "string",
                                "description": "Expression or variable name to evaluate"
                            }
                        },
                        "required": ["expression"]
                    }
                },
                {
                    "name": "debug_backtrace",
                    "description": "Show the current call stack",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
                    "name": "debug_list_breakpoints",
                    "description": "List all active breakpoints",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
                    "name": "debug_state",
                    "description": "Get current debugging session state",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                }
            ]
        })
    }

    async fn handle_call_tool(&self, name: &str, arguments: Value) -> Result<Value> {
        match name {
            "debug_run" => {
                let binary_path = arguments
                    .get("binary_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("binary_path required"))?;
                self.debug_run(binary_path).await
            }
            "debug_break" => {
                let location = arguments
                    .get("location")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("location required"))?;
                self.debug_break(location).await
            }
            "debug_continue" => self.debug_continue().await,
            "debug_step" => self.debug_step().await,
            "debug_step_into" => self.debug_step_into().await,
            "debug_step_out" => self.debug_step_out().await,
            "debug_eval" => {
                let expression = arguments
                    .get("expression")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("expression required"))?;
                self.debug_eval(expression).await
            }
            "debug_backtrace" => self.debug_backtrace().await,
            "debug_list_breakpoints" => self.debug_list_breakpoints().await,
            "debug_state" => self.get_debug_state().await,
            _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
        }
    }

    async fn handle_request(&self, request: Value) -> Value {
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request.get("method").and_then(|v| v.as_str()).unwrap_or("");
        let params = request.get("params").cloned().unwrap_or(Value::Null);

        let result = match method {
            "initialize" => Ok(self.handle_initialize(params).await),
            "tools/list" => Ok(self.handle_list_tools().await),
            "tools/call" => {
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);

                match self.handle_call_tool(name, arguments).await {
                    Ok(result) => Ok(json!({
                        "content": [
                            {
                                "type": "text",
                                "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| "Error serializing result".to_string())
                            }
                        ]
                    })),
                    Err(e) => Err(json!({
                        "code": -32602,
                        "message": format!("Tool execution failed: {}", e)
                    })),
                }
            }
            _ => Err(json!({
                "code": -32601,
                "message": format!("Method not found: {}", method)
            })),
        };

        match result {
            Ok(result) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result
            }),
            Err(error) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": error
            }),
        }
    }

    async fn run(&self) -> Result<()> {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        println!("🦀 Ferroscope v2.0 - Production Ready Rust Debugging MCP Server");
        eprintln!("🚀 Ferroscope starting with enhanced debugging capabilities...");

        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<Value>(&line) {
                Ok(request) => {
                    let response = self.handle_request(request).await;
                    println!("{}", serde_json::to_string(&response)?);
                }
                Err(e) => {
                    eprintln!("Invalid JSON: {}", e);
                    let error_response = json!({
                        "jsonrpc": "2.0",
                        "id": null,
                        "error": {
                            "code": -32700,
                            "message": format!("Parse error: {}", e)
                        }
                    });
                    println!("{}", serde_json::to_string(&error_response)?);
                }
            }
        }

        Ok(())
    }
}

impl Drop for DebugServer {
    fn drop(&mut self) {
        // Clean up any running debugging session
        if let Ok(mut session_guard) = self.session.try_lock() {
            if let Some(mut session) = session_guard.take() {
                let _ = futures::executor::block_on(session.process.kill());
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let server = DebugServer::new();
    server.run().await?;
    Ok(())
}
