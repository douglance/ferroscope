# Ferroscope

**🎯 AI agents can now debug Rust programs as intuitively as writing a single character.**

A **pure Rust** implementation of a Model Context Protocol (MCP) server that provides AI agents with debugging capabilities through simple JSON-RPC commands over stdio.

## Features

✅ **Real debugging**: Actual LLDB/GDB integration, not mock responses  
✅ **Cross-platform**: Works with LLDB (macOS) and GDB (Linux)  
✅ **10 debugging tools**: Complete debugging workflow  
✅ **Global installation**: Use anywhere with `cargo install ferroscope`  
✅ **Claude Code ready**: Includes MCP configuration  

## Installation

```bash
# Install from crates.io
cargo install ferroscope

# Or build from source
git clone https://github.com/douglance/ferroscope.git
cd ferroscope
cargo install --path .
```

## Usage

Once installed, ferroscope runs as an MCP server that communicates via JSON-RPC over stdio:

```bash
# Run the MCP server
ferroscope

# Test with a simple command
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | ferroscope
```

## Claude Code Integration

Add this to your Claude Code MCP settings:

```json
{
  "mcpServers": {
    "rust-debugger": {
      "command": "ferroscope",
      "args": [],
      "env": {},
      "description": "Rust debugging tools via LLDB/GDB"
    }
  }
}
```

Then restart Claude Code and ask: *"Debug this Rust program"*

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

## JSON-RPC Usage

```bash
# Initialize and debug a program
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"debug_run","arguments":{"binary_path":"./my_project"}}}' | ferroscope
```

## Requirements

- Rust toolchain
- LLDB (macOS) or GDB (Linux)

## Verification

After installation, verify ferroscope is working:

```bash
# Check it's installed
which ferroscope

# Run the comprehensive test suite (if built from source)
cargo run --bin comprehensive-test
```

## What You Get

After installing ferroscope, you'll have:
- A global `ferroscope` command available in your terminal
- Full LLDB/GDB debugging capabilities accessible via JSON-RPC
- Claude Code integration for AI-powered debugging
- Support for all major Rust debugging workflows

**Ready to debug Rust programs with AI!** 🦀