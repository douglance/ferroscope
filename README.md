# Ferroscope

MCP server that enables AI assistants to debug Rust programs using LLDB and GDB.

## Quick Start

### 1. Install

```bash
cargo install ferroscope
```

<details>
<summary>Alternative: Build from source</summary>

```bash
git clone https://github.com/douglance/ferroscope.git
cd ferroscope
cargo install --path .
```
</details>

### 2. Configure Your AI Assistant

Add this to your AI assistant's MCP settings:

```json
{
  "mcpServers": {
    "ferroscope": {
      "command": "ferroscope",
      "args": [],
      "env": {},
      "description": "Rust debugging via LLDB/GDB"
    }
  }
}
```

### 3. Add Custom Instructions

Copy this snippet to your AI assistant's custom instructions:

```markdown
When debugging Rust programs, use ferroscope with this workflow:
1. Load program: debug_run /path/to/project
2. Set breakpoints: debug_break main or debug_break src/main.rs:25
3. Start execution: debug_continue
4. At breakpoints: debug_eval variable_name to inspect values
5. Step through: debug_step (over), debug_step_into (into), debug_step_out (out)
6. Check state: debug_state to see current status
7. View stack: debug_backtrace when errors occur

Always start with debug_run, then set breakpoints before debug_continue.
```

### 4. Start Debugging

Ask your AI assistant: "Debug this Rust program" and it will use ferroscope automatically.

## Configuration by AI Assistant

<details>
<summary><strong>Claude Code</strong></summary>

The configuration above works for Claude Code. Add it to Settings → MCP Servers, then restart.

</details>

<details>
<summary><strong>Cursor</strong></summary>

Add to `.cursor/config.json`:
```json
{"tools": {"ferroscope": {"command": "ferroscope", "description": "Debug Rust programs"}}}
```

</details>

<details>
<summary><strong>Windsurf</strong></summary>

Add to tools configuration:
```json
{"customTools": [{"name": "ferroscope", "command": "ferroscope", "type": "mcp"}]}
```

</details>

<details>
<summary><strong>Zed</strong></summary>

Add to `~/.config/zed/settings.json`:
```json
{"assistant": {"tools": {"ferroscope": {"command": "ferroscope", "args": []}}}}
```

</details>

<details>
<summary><strong>Other AI Assistants</strong></summary>

For any MCP-compatible AI assistant, the command is `ferroscope`. Check your assistant's documentation for "MCP tools" or "external tools".

</details>

## Available Tools

1. **`debug_run`** - Load and prepare Rust programs for debugging
2. **`debug_break`** - Set breakpoints at functions or lines  
3. **`debug_continue`** - Launch/continue program execution
4. **`debug_step`** - Step through code line by line
5. **`debug_step_into`** - Step into function calls
6. **`debug_step_out`** - Step out of current function
7. **`debug_eval`** - Evaluate expressions and inspect variables
8. **`debug_backtrace`** - Show call stack
9. **`debug_list_breakpoints`** - List all breakpoints
10. **`debug_state`** - Get current debugging session state

## Requirements

- Rust toolchain
- LLDB (macOS) or GDB (Linux)
- Windows not currently supported (WinDbg integration planned)

## Verification

Check ferroscope is installed and working:

```bash
# Verify installation
which ferroscope

# Test basic functionality
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | ferroscope

# Run test suite (if built from source)
cargo run --bin comprehensive-test
# Expected output:
# 🧪 FERROSCOPE COMPREHENSIVE TEST SUITE
# ✅ MCP Protocol Test: PASSED
# ✅ Program Loading Test: PASSED
# ✅ Breakpoint Test: PASSED
# ... (more tests)
# 🎉 ALL TESTS PASSED! Ferroscope functionality verified!
```

## Limitations

- **Security**: Currently runs with full user privileges without sandboxing. Only use with trusted code.
- **Platform Support**: Windows is not supported (WinDbg integration planned)
- **Performance**: No connection pooling or resource limits for concurrent debugging sessions
- **Error Recovery**: Limited error handling for malformed debugger output
- **Binary Types**: Only supports Rust binaries compiled with debug symbols
- **Debugger Versions**: Tested with LLDB 15+ and GDB 12+

---

**Ready to debug Rust programs with AI!** 🦀