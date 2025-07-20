# Rust Debugger MCP Server Best Practices

## Context Optimization Guidelines

### 1. Verbosity Level Selection

Choose the appropriate verbosity level based on your debugging needs:

```yaml
minimal:      # Use for: Quick status checks, CI/CD pipelines
focused:      # Use for: Normal debugging sessions (RECOMMENDED DEFAULT)
standard:     # Use for: Complex issues requiring full debugger output
comprehensive: # Use for: Deep debugging with maximum context
```

### 2. Efficient Debugging Workflow

```markdown
1. START WITH FOCUSED VERBOSITY
   - Begin debugging with verbosity="focused" to maintain reasonable context size
   - Only escalate to "standard" or "comprehensive" when specific issues require it

2. USE TARGETED INSPECTION
   - Specify focus_areas to reduce output noise:
     focus_areas: ["variables", "stack"]  # Only get variable state and stack trace
   
3. ITERATIVE REFINEMENT
   - Start broad, then narrow:
     a) First call: verbosity="minimal" to understand state
     b) Second call: verbosity="focused" with specific focus_areas
     c) Final call: verbosity="standard" only for problematic areas
```

### 3. Parameter Usage Patterns

#### Optimal for Variable Inspection
```json
{
  "expression": "my_variable",
  "verbosity": "focused",
  "focus_areas": ["variables"]
}
```

#### Optimal for Stack Analysis
```json
{
  "verbosity": "focused",
  "focus_areas": ["stack", "location"]
}
```

#### Optimal for Breakpoint Management
```json
{
  "verbosity": "minimal",
  "focus_areas": ["state"]
}
```

### 4. Context Size Management

**DO:**
- Use `minimal` verbosity for state checks and flow control
- Parse output programmatically when possible
- Request specific focus_areas rather than full output
- Cache debugging state between calls to avoid redundant queries

**DON'T:**
- Use `comprehensive` verbosity unless absolutely necessary
- Request all focus_areas simultaneously
- Repeatedly query the same information with high verbosity
- Include raw debugger output in summaries

### 5. Debugging State Efficiency

```markdown
EFFICIENT PATTERN:
1. debug_run (verbosity="minimal")
2. debug_break (verbosity="minimal") 
3. debug_continue (verbosity="focused", focus_areas=["location"])
4. debug_eval (verbosity="focused", focus_areas=["variables"])

INEFFICIENT PATTERN:
1. debug_run (verbosity="comprehensive")  # Too much initial context
2. debug_state (verbosity="standard")     # Redundant state query
3. debug_eval (verbosity="comprehensive") # Excessive for simple variable check
```

### 6. Advanced Patterns

#### Pattern 1: Conditional Verbosity Escalation
```python
# Pseudocode for adaptive debugging
result = debug_step(verbosity="minimal")
if result.needs_more_context:
    result = debug_step(verbosity="focused", focus_areas=["stack", "variables"])
if still_unclear:
    result = debug_step(verbosity="standard")
```

#### Pattern 2: Context Caching
```python
# Cache frequently accessed state
cached_state = debug_state(verbosity="minimal")
# Use cached state for multiple operations without re-querying
```

#### Pattern 3: Batch Operations
```python
# Group related operations to minimize context switches
breakpoints = [
    {"location": "main.rs:10", "verbosity": "minimal"},
    {"location": "lib.rs:25", "verbosity": "minimal"}
]
# Set all breakpoints with minimal output, then query state once
```

### 7. Performance Considerations

- **Minimal verbosity**: ~100-200 bytes per response
- **Focused verbosity**: ~300-800 bytes per response  
- **Standard verbosity**: ~1-5KB per response
- **Comprehensive verbosity**: ~5-20KB per response

**Target**: Keep average response size under 1KB for optimal LLM context usage.

### 8. Error Handling Best Practices

```markdown
1. ALWAYS use focused/standard verbosity when errors occur
2. Include error-specific focus_areas: ["output", "state"]
3. Capture full context only for first occurrence of an error type
4. Use minimal verbosity for error recovery confirmation
```

### 9. Integration Guidelines

When integrating with LLM workflows:

```yaml
For Planning Phase:
  verbosity: minimal
  focus_areas: [state]

For Implementation Phase:
  verbosity: focused
  focus_areas: [location, variables]

For Debugging Phase:
  verbosity: standard
  focus_areas: [output, stack, variables]

For Verification Phase:
  verbosity: minimal
  focus_areas: [state]
```

### 10. Quick Reference

| Task | Verbosity | Focus Areas | Rationale |
|------|-----------|-------------|-----------|
| Initial setup | minimal | state | Reduce startup noise |
| Setting breakpoints | minimal | state | Confirmation only needed |
| Variable inspection | focused | variables | Targeted data extraction |
| Stack trace analysis | focused | stack, location | Structured over raw |
| Error investigation | standard | output, state | Full context for diagnosis |
| State verification | minimal | state | Quick status check |
| Complex debugging | comprehensive | all | Last resort for difficult bugs |

## Summary

The key to effective debugging with minimal context overhead is:
1. Start with the lowest verbosity that meets your needs
2. Use focus_areas to target specific information
3. Escalate verbosity only when necessary
4. Cache and reuse debugging state when possible
5. Prefer structured output over raw debugger dumps

Remember: **Less context with higher relevance beats more context with noise.**