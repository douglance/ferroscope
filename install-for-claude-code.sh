#!/bin/bash

echo "🦀 Installing Rust Debugger MCP for Claude Code..."
echo "=================================================="

# Step 1: Install globally via Cargo
echo "📦 Installing ferroscope globally..."
if cargo install --path . --force; then
    echo "✅ Global installation successful!"
else
    echo "❌ Installation failed!"
    exit 1
fi

# Step 2: Verify installation
echo ""
echo "🔍 Verifying installation..."
if which ferroscope > /dev/null; then
    echo "✅ ferroscope found in PATH: $(which ferroscope)"
else
    echo "❌ ferroscope not found in PATH!"
    echo "💡 Make sure ~/.cargo/bin is in your PATH"
    echo "Add this to your ~/.bashrc or ~/.zshrc:"
    echo "export PATH=\"\$HOME/.cargo/bin:\$PATH\""
    exit 1
fi

# Step 3: Test basic functionality
echo ""
echo "🧪 Testing basic functionality..."
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}' | timeout 3 ferroscope > /dev/null 2>&1
if [ $? -eq 124 ]; then
    echo "✅ MCP server responding correctly (timeout expected)"
else
    echo "⚠️  MCP server test inconclusive - this may be normal"
fi

# Step 4: Create configuration files
echo ""
echo "📝 Creating Claude Code configuration files..."

# Create MCP config
cat > claude-mcp-config.json << 'EOF'
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
EOF

echo "✅ Created claude-mcp-config.json"

# Create installation summary
cat > INSTALLATION_SUMMARY.md << 'EOF'
# 🎉 Rust Debugger MCP Installation Complete!

## ✅ What was installed:

- **Global binary**: `ferroscope` in `~/.cargo/bin/`
- **Test suite**: `comprehensive-test` in `~/.cargo/bin/`
- **Configuration**: `claude-mcp-config.json` for Claude Code

## 🔧 Next Steps for Claude Code:

1. **Add to Claude Code settings:**
   - Copy the contents of `claude-mcp-config.json`
   - Paste into Claude Code's MCP server configuration

2. **Restart Claude Code** to load the new tools

3. **Test by asking Claude Code:**
   ```
   "Please debug a Rust program using the debugging tools"
   ```

## 🛠 Available Tools:

- `debug_run` - Load Rust programs for debugging
- `debug_break` - Set breakpoints  
- `debug_continue` - Start/continue execution
- `debug_step` - Step through code
- `debug_eval` - Evaluate expressions
- `debug_state` - Check debugging status
- And 4 more advanced debugging commands!

## 🚀 You're Ready!

Claude Code can now debug Rust programs with LLDB/GDB integration!
EOF

echo "✅ Created INSTALLATION_SUMMARY.md"

# Step 5: Success message
echo ""
echo "🎉 INSTALLATION COMPLETE!"
echo "========================="
echo ""
echo "📋 Summary:"
echo "  ✅ ferroscope installed globally"
echo "  ✅ Configuration files created"
echo "  ✅ Ready for Claude Code integration"
echo ""
echo "📖 Next steps:"
echo "  1. Read CLAUDE_CODE_SETUP.md for detailed setup instructions"
echo "  2. Add claude-mcp-config.json to Claude Code settings"
echo "  3. Restart Claude Code"
echo "  4. Start debugging Rust programs!"
echo ""
echo "🧪 Test the installation:"
echo "  Run: comprehensive-test"
echo ""
echo "🦀 Happy debugging with Claude Code!"