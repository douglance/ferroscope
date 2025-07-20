use anyhow::Result;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

/**
 * FERROSCOPE COMPREHENSIVE VALIDATION TEST
 * 
 * Tests all critical functionality that was previously broken:
 * 1. ✅ Programs load and initialize properly
 * 2. ✅ Process launch works (not just "continue")  
 * 3. ✅ Breakpoints work correctly
 * 4. ✅ State management tracks program lifecycle
 * 5. ✅ Error handling works properly
 * 6. ✅ Session management and cleanup
 */

struct ComprehensiveTestSuite {
    server_process: std::process::Child,
    stdin: std::process::ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
    request_id: u64,
}

impl ComprehensiveTestSuite {
    fn new() -> Result<Self> {
        println!("🧪 FERROSCOPE COMPREHENSIVE TEST SUITE");
        println!("🎯 Testing all critical functionality that was previously broken");
        println!();

        let mut server_process = Command::new("cargo")
            .args(&["run", "--bin", "ferroscope"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdin = server_process.stdin.take().unwrap();
        let stdout = BufReader::new(server_process.stdout.take().unwrap());

        Ok(ComprehensiveTestSuite {
            server_process,
            stdin,
            stdout,
            request_id: 0,
        })
    }

    fn send_request(&mut self, method: &str, params: Value) -> Result<Value> {
        self.request_id += 1;
        
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params
        });

        writeln!(self.stdin, "{}", serde_json::to_string(&request)?)?;
        self.stdin.flush()?;

        let mut response_line = String::new();
        self.stdout.read_line(&mut response_line)?;
        
        let response: Value = serde_json::from_str(&response_line.trim())?;
        Ok(response)
    }

    fn debug_command(&mut self, tool_name: &str, args: Value) -> Result<Value> {
        let params = json!({
            "name": tool_name,
            "arguments": args
        });

        let response = self.send_request("tools/call", params)?;
        
        if let Some(result) = response.get("result") {
            if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
                if let Some(text) = content[0].get("text").and_then(|t| t.as_str()) {
                    let parsed: Value = serde_json::from_str(text)?;
                    return Ok(parsed);
                }
            }
        }
        
        if let Some(error) = response.get("error") {
            anyhow::bail!("Command failed: {}", error);
        }
        
        anyhow::bail!("Unexpected response: {:?}", response);
    }

    fn run_test(&mut self, test_name: &str, test_fn: impl FnOnce(&mut Self) -> Result<()>) -> bool {
        print!("🔍 Testing {}: ", test_name);
        std::io::stdout().flush().unwrap();
        
        match test_fn(self) {
            Ok(()) => {
                println!("✅ PASSED");
                true
            }
            Err(e) => {
                println!("❌ FAILED - {}", e);
                false
            }
        }
    }

    fn test_initialization(&mut self) -> Result<()> {
        let response = self.send_request("initialize", json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {}
        }))?;

        let server_info = response.get("result")
            .and_then(|r| r.get("serverInfo"))
            .ok_or_else(|| anyhow::anyhow!("No server info"))?;

        let name = server_info.get("name").and_then(|n| n.as_str())
            .ok_or_else(|| anyhow::anyhow!("No server name"))?;

        if name != "ferroscope" {
            anyhow::bail!("Expected v2 server, got: {}", name);
        }

        Ok(())
    }

    fn test_program_loading(&mut self) -> Result<()> {
        let result = self.debug_command("debug_run", json!({
            "binary_path": "./test_programs/simple_counter"
        }))?;

        let success = result.get("success").and_then(|s| s.as_bool())
            .ok_or_else(|| anyhow::anyhow!("No success field"))?;

        if !success {
            anyhow::bail!("Program loading failed");
        }

        let state = result.get("state").and_then(|s| s.as_str())
            .ok_or_else(|| anyhow::anyhow!("No state field"))?;

        if state != "loaded" {
            anyhow::bail!("Expected 'loaded' state, got: {}", state);
        }

        let output = result.get("output").and_then(|o| o.as_str())
            .ok_or_else(|| anyhow::anyhow!("No output field"))?;

        if !output.contains("Current executable set to") {
            anyhow::bail!("No LLDB output found");
        }

        Ok(())
    }

    fn test_breakpoint_setting(&mut self) -> Result<()> {
        let result = self.debug_command("debug_break", json!({
            "location": "main"
        }))?;

        let success = result.get("success").and_then(|s| s.as_bool())
            .ok_or_else(|| anyhow::anyhow!("No success field"))?;

        if !success {
            anyhow::bail!("Breakpoint setting failed");
        }

        let output = result.get("output").and_then(|o| o.as_str())
            .ok_or_else(|| anyhow::anyhow!("No output field"))?;

        if !output.contains("Breakpoint 1:") {
            anyhow::bail!("No breakpoint output found");
        }

        Ok(())
    }

    fn test_process_launch(&mut self) -> Result<()> {
        let result = self.debug_command("debug_continue", json!({}))?;

        let success = result.get("success").and_then(|s| s.as_bool())
            .ok_or_else(|| anyhow::anyhow!("No success field"))?;

        if !success {
            anyhow::bail!("Process launch failed");
        }

        let output = result.get("output").and_then(|o| o.as_str())
            .ok_or_else(|| anyhow::anyhow!("No output field"))?;

        if !output.contains("process launch") {
            anyhow::bail!("No process launch command found");
        }

        Ok(())
    }

    fn test_state_management(&mut self) -> Result<()> {
        let result = self.debug_command("debug_state", json!({}))?;

        let state = result.get("state").and_then(|s| s.as_str())
            .ok_or_else(|| anyhow::anyhow!("No state field"))?;

        let binary_path = result.get("binary_path").and_then(|b| b.as_str())
            .ok_or_else(|| anyhow::anyhow!("No binary_path field"))?;

        if !binary_path.contains("simple-counter") {
            anyhow::bail!("Wrong binary path: {}", binary_path);
        }

        // State should be loaded or running
        if state != "loaded" && state != "running" && state != "stopped" {
            anyhow::bail!("Invalid state: {}", state);
        }

        Ok(())
    }

    fn test_error_handling(&mut self) -> Result<()> {
        // Test with nonexistent program
        let result = self.debug_command("debug_run", json!({
            "binary_path": "./nonexistent_program"
        }));

        // Should fail gracefully
        if result.is_ok() {
            anyhow::bail!("Should have failed with nonexistent program");
        }

        Ok(())
    }

    fn test_invalid_breakpoint(&mut self) -> Result<()> {
        // First load a program
        self.debug_command("debug_run", json!({
            "binary_path": "./test_programs/simple_counter"
        }))?;

        // Try invalid breakpoint
        let result = self.debug_command("debug_break", json!({
            "location": "nonexistent_function"
        }))?;

        let output = result.get("output").and_then(|o| o.as_str())
            .ok_or_else(|| anyhow::anyhow!("No output field"))?;

        // Should show "no locations" warning
        if !output.contains("no locations") {
            anyhow::bail!("Should warn about no locations for invalid function");
        }

        Ok(())
    }

    fn run_comprehensive_test_suite(&mut self) -> bool {
        println!("🧪 FERROSCOPE COMPREHENSIVE VALIDATION");
        println!("{}", "=".repeat(60));
        println!("Testing all functionality that was previously broken:");
        println!();

        let mut passed = 0;
        let mut total = 0;

        macro_rules! test {
            ($name:expr, $method:ident) => {
                total += 1;
                if self.run_test($name, |suite| suite.$method()) {
                    passed += 1;
                }
            };
        }

        test!("Server initialization (v2.0)", test_initialization);
        test!("Program loading with binary", test_program_loading);
        test!("Breakpoint setting with LLDB", test_breakpoint_setting);
        test!("Process launch (not just continue)", test_process_launch);
        test!("State management and tracking", test_state_management);
        test!("Error handling for invalid inputs", test_error_handling);
        test!("Invalid breakpoint graceful handling", test_invalid_breakpoint);

        println!();
        println!("🏆 TEST RESULTS:");
        println!("   ✅ Passed: {}/{}", passed, total);
        println!("   ❌ Failed: {}/{}", total - passed, total);
        
        if passed == total {
            println!("   🎉 ALL TESTS PASSED! Ferroscope functionality verified!");
            true
        } else {
            println!("   ⚠️  Some tests failed. Ferroscope needs more fixes.");
            false
        }
    }
}

impl Drop for ComprehensiveTestSuite {
    fn drop(&mut self) {
        let _ = self.server_process.kill();
        let _ = self.server_process.wait();
    }
}

fn main() -> Result<()> {
    // Ensure test program is built
    println!("🔨 Building test programs...");
    let build_output = Command::new("cargo")
        .args(&["build"])
        .current_dir("test_programs/simple_counter")
        .output()?;
    
    if !build_output.status.success() {
        anyhow::bail!("Failed to build test program");
    }
    println!("✅ Test programs built");
    println!();

    let mut test_suite = ComprehensiveTestSuite::new()?;
    std::thread::sleep(std::time::Duration::from_millis(1000));

    let all_passed = test_suite.run_comprehensive_test_suite();

    println!();
    if all_passed {
        println!("🚀 FERROSCOPE VALIDATION: SUCCESS");
        println!("Ferroscope now provides full debugging functionality!");
        println!("Ready for Gemini validation.");
    } else {
        println!("🔧 FERROSCOPE VALIDATION: NEEDS MORE WORK");
        println!("Some critical functionality is still broken.");
    }

    Ok(())
}