# Contributing to Ferroscope

## Automated Release System

Ferroscope uses automated semantic versioning and publishing to crates.io based on conventional commit messages.

### Conventional Commits

Use these commit message formats to trigger automatic releases:

#### Patch Release (0.1.1 → 0.1.2)
```bash
fix: resolve clippy warnings in CI
fix(debugger): handle LLDB timeout correctly
```

#### Minor Release (0.1.1 → 0.2.0)
```bash
feat: add GDB support for Linux debugging
feat(eval): support complex expression evaluation
```

#### Major Release (0.1.1 → 1.0.0)
```bash
feat!: redesign MCP protocol interface
fix!: change breakpoint API (BREAKING CHANGE)

BREAKING CHANGE: The debug_break API now requires location type specification
```

### Manual Release

You can also trigger releases manually via GitHub Actions:

1. Go to GitHub Actions → "Release and Publish"
2. Click "Run workflow"
3. Select version type: `patch`, `minor`, `major`, or specific version like `1.2.3`

### What Happens During Release

1. **Trigger Detection**: Commit messages are scanned for conventional commit patterns
2. **Full Test Suite**: All tests, clippy, and formatting checks must pass
3. **Version Bump**: `cargo-release` updates version in Cargo.toml and tags the commit
4. **Publish**: Package is published to crates.io automatically
5. **GitHub Release**: Release notes are generated and published
6. **Notification**: Success/failure status is reported

### Development Workflow

```bash
# Work on a feature
git checkout -b feature/new-debugging-tool

# Make changes with conventional commits
git commit -m "feat: add memory inspection tool"
git commit -m "fix: handle invalid memory addresses"
git commit -m "docs: update tool usage examples"

# Push and create PR
git push origin feature/new-debugging-tool

# After PR merge to master, automatic release will trigger if commits warrant it
```

### Release Requirements

Before any release is published:

- ✅ All tests must pass on Linux and macOS
- ✅ Code must pass `cargo fmt --check`
- ✅ Code must pass `cargo clippy` with no warnings
- ✅ Examples must build successfully
- ✅ `cargo publish --dry-run` must succeed

### Secrets Required

The following GitHub secrets must be configured for automated publishing:

- `CARGO_REGISTRY_TOKEN`: Token from crates.io for publishing packages

### Version Strategy

Ferroscope follows [Semantic Versioning](https://semver.org/):

- **MAJOR** version when you make incompatible API changes
- **MINOR** version when you add functionality in a backwards compatible manner  
- **PATCH** version when you make backwards compatible bug fixes

### Commit Message Examples

```bash
# Patch releases
git commit -m "fix: resolve memory leak in debugger session cleanup"
git commit -m "fix(ci): update Rust version to match local environment"

# Minor releases  
git commit -m "feat: add support for conditional breakpoints"
git commit -m "feat(eval): implement variable watching functionality"

# Major releases
git commit -m "feat!: redesign tool interface for MCP 2.0"
git commit -m "fix!: change tool names for consistency

BREAKING CHANGE: All tool names now use snake_case instead of camelCase"

# Non-release commits (no version bump)
git commit -m "docs: update README with new installation instructions"
git commit -m "chore: update dependencies"
git commit -m "test: add integration tests for error handling"
git commit -m "refactor: simplify session management code"
```

### Testing Releases

To test the release process without publishing:

1. Create a branch from master
2. Make a commit with conventional format
3. Push to GitHub and observe the workflow in Actions tab
4. The workflow will run all checks but won't actually publish since it's not on master

### Troubleshooting

**Release not triggering:**
- Check commit message follows conventional commit format exactly
- Ensure push is to `master` branch
- Check GitHub Actions logs for workflow execution

**Publish fails:**
- Verify `CARGO_REGISTRY_TOKEN` secret is set correctly
- Check if version already exists on crates.io
- Review crates.io API limits and status

**Tests fail:**
- All tests must pass before release
- Check CI logs for specific test failures
- Fix issues and push new commits to trigger re-release