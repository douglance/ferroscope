# Ferroscope TODO List

This document tracks critical issues identified during code review, prioritized by severity.

## 🔴 Critical Security Issues (HIGH PRIORITY)

### 1. Command Injection Vulnerability
**Issue**: User-provided `binary_path` is passed directly to shell commands without validation
**Risk**: Arbitrary command execution with user privileges
**Fix**: 
- Implement strict path validation and sanitization
- Use `Command::arg()` instead of string interpolation
- Whitelist allowed characters in paths

### 2. Arbitrary Code Execution via debug_eval
**Issue**: `debug_eval` passes user expressions directly to LLDB/GDB
**Risk**: Attackers can execute arbitrary system commands through debugger
**Fix**:
- Implement expression sandboxing
- Create allowlist of safe debugger commands
- Add expression complexity limits

### 3. No Process Isolation
**Issue**: Debugger runs with full user privileges, no sandboxing
**Risk**: Complete system access if exploited
**Fix**:
- Implement process sandboxing (e.g., using nix crate)
- Run debugger in restricted environment
- Add capability dropping after initialization

## 🟡 Architecture & Code Quality Issues (MEDIUM PRIORITY)

### 4. Monolithic Architecture
**Issue**: Single 800+ line main.rs file
**Impact**: Poor maintainability, hard to test, difficult to understand
**Fix**:
- Break into modules: `protocol/`, `debugger/`, `security/`, `error/`
- Separate concerns: MCP handling, debugger interaction, process management
- Create trait-based abstractions for debugger backends

### 5. Poor Error Handling
**Issue**: String-based errors, loss of error context
**Impact**: Difficult debugging, poor error recovery
**Fix**:
- Create custom error types with `thiserror`
- Implement proper error propagation
- Add structured error responses to clients

### 6. No Resource Management
**Issue**: Spawned processes not tracked, no cleanup
**Impact**: Resource leaks, zombie processes
**Fix**:
- Implement process registry
- Add proper cleanup on drop
- Track and limit concurrent debugging sessions

## 🟠 Performance & Scalability Issues (MEDIUM PRIORITY)

### 7. Missing Async Cancellation
**Issue**: No timeouts or cancellation for long-running operations
**Impact**: Hung processes, resource exhaustion
**Fix**:
- Add configurable timeouts for all operations
- Implement proper async cancellation with tokio
- Add request ID tracking for cancellation

### 8. No Connection Pooling
**Issue**: Each request could spawn new debugger process
**Impact**: Poor performance, resource exhaustion
**Fix**:
- Implement debugger process pool
- Reuse debugger sessions where possible
- Add connection limits

### 9. Unbounded Resource Usage
**Issue**: No limits on concurrent operations or memory usage
**Impact**: DoS vulnerability, system instability
**Fix**:
- Add rate limiting
- Implement memory usage caps
- Limit concurrent debugging sessions

## 🟢 Additional Improvements (LOW PRIORITY)

### 10. Testing Infrastructure
**Issue**: Limited test coverage, no unit tests
**Fix**:
- Add unit tests for each module
- Implement integration tests
- Add property-based testing for security-critical paths

### 11. Documentation
**Issue**: No API documentation, security guidelines missing
**Fix**:
- Add comprehensive rustdoc comments
- Create security best practices guide
- Document threat model

### 12. Platform Support
**Issue**: No Windows support
**Fix**:
- Add WinDbg backend
- Abstract debugger interface
- Add platform-specific tests

## Implementation Priority

1. **Phase 1 (Security Critical)**: Issues #1, #2, #3
2. **Phase 2 (Architecture)**: Issues #4, #5, #6  
3. **Phase 3 (Reliability)**: Issues #7, #8, #9
4. **Phase 4 (Polish)**: Issues #10, #11, #12

## Notes

- All security issues should be addressed before any public release
- Consider adding a security policy and responsible disclosure process
- Regular security audits recommended given the nature of the tool