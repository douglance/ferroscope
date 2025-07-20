#!/bin/bash
set -e

# Test release process locally without actually publishing
# Usage: ./scripts/test-release.sh [patch|minor|major|1.2.3]

RELEASE_TYPE=${1:-patch}

echo "🧪 Testing release process locally..."
echo "Release type: $RELEASE_TYPE"
echo ""

# Check if cargo-release is installed
if ! command -v cargo-release &> /dev/null; then
    echo "❌ cargo-release not found. Installing..."
    cargo install cargo-release
fi

# Run all the same checks as CI
echo "🔍 Running pre-release checks..."

echo "  📋 Checking formatting..."
cargo fmt --all -- --check

echo "  📋 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

echo "  📋 Building project..."
cargo build --verbose

echo "  📋 Running tests..."
cargo test --verbose

echo "  📋 Building examples..."
(cd examples/simple_counter && cargo build)

echo "  📋 Testing publish (dry run)..."
cargo publish --dry-run

echo ""
echo "✅ All checks passed!"
echo ""

# Show what would be released
echo "🏷️  Current version: $(cargo pkgid | cut -d# -f2 | cut -d: -f2)"

# Show what the new version would be
if [[ "$RELEASE_TYPE" =~ ^[0-9]+\.[0-9]+\.[0-9]+.*$ ]]; then
    NEW_VERSION="$RELEASE_TYPE"
else
    # Calculate what the new version would be
    CURRENT_VERSION=$(cargo pkgid | cut -d# -f2 | cut -d: -f2)
    IFS='.' read -ra VERSION_PARTS <<< "$CURRENT_VERSION"
    MAJOR=${VERSION_PARTS[0]}
    MINOR=${VERSION_PARTS[1]}
    PATCH=${VERSION_PARTS[2]}
    
    case $RELEASE_TYPE in
        major)
            NEW_VERSION="$((MAJOR + 1)).0.0"
            ;;
        minor)
            NEW_VERSION="$MAJOR.$((MINOR + 1)).0"
            ;;
        patch)
            NEW_VERSION="$MAJOR.$MINOR.$((PATCH + 1))"
            ;;
        *)
            echo "❌ Unknown release type: $RELEASE_TYPE"
            exit 1
            ;;
    esac
fi

echo "🚀 Would release version: $NEW_VERSION"
echo ""

echo "📝 To actually release, commit with a conventional commit message:"
case $RELEASE_TYPE in
    major)
        echo "   git commit -m 'feat!: breaking change description'"
        ;;
    minor)
        echo "   git commit -m 'feat: new feature description'"
        ;;
    patch)
        echo "   git commit -m 'fix: bug fix description'"
        ;;
esac

echo ""
echo "📝 Or manually trigger release via GitHub Actions:"
echo "   Go to: https://github.com/douglance/ferroscope/actions/workflows/release.yml"
echo "   Click 'Run workflow' and select version type"
echo ""
echo "✨ Test completed successfully!"