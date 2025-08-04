#!/bin/bash
#
# Setup script for installing Git pre-commit hooks
# Run this script after cloning the repository to enable pre-commit checks
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "${BLUE}🔧 Setting up Git pre-commit hooks for rs-mock-server...${NC}"

# Check if we're in a Git repository
if [ ! -d ".git" ]; then
    echo "${RED}❌ This directory is not a Git repository!${NC}"
    echo "${YELLOW}💡 Please run this script from the root of the rs-mock-server repository.${NC}"
    exit 1
fi

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "${RED}❌ Cargo not found! Please install Rust and Cargo first.${NC}"
    exit 1
fi

# Create the pre-commit hook
PRE_COMMIT_HOOK=".git/hooks/pre-commit"

cat > "$PRE_COMMIT_HOOK" << 'EOF'
#!/bin/sh
#
# Pre-commit hook for rs-mock-server
# This hook runs Cargo tests before allowing a commit.
# If any tests fail, the commit will be aborted.
#

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
RUN_TESTS=true
RUN_CLIPPY=false
RUN_FORMAT_CHECK=false

echo "${BLUE}🧪 Running pre-commit checks...${NC}"

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "${RED}❌ Cargo not found! Please install Rust and Cargo.${NC}"
    exit 1
fi

# Function to run tests
run_tests() {
    echo "${BLUE}🧪 Running tests...${NC}"
    if cargo test --quiet; then
        echo "${GREEN}✅ All tests passed!${NC}"
        return 0
    else
        echo "${RED}❌ Tests failed!${NC}"
        echo "${YELLOW}💡 Run 'cargo test' to see detailed test results.${NC}"
        return 1
    fi
}

# Function to run clippy
run_clippy() {
    echo "${BLUE}🔍 Running Clippy...${NC}"
    if cargo clippy --all-targets --all-features -- -D warnings; then
        echo "${GREEN}✅ Clippy checks passed!${NC}"
        return 0
    else
        echo "${RED}❌ Clippy found issues!${NC}"
        echo "${YELLOW}💡 Run 'cargo clippy --all-targets --all-features -- -D warnings' to see details.${NC}"
        return 1
    fi
}

# Function to check formatting
run_format_check() {
    echo "${BLUE}📝 Checking code formatting...${NC}"
    if cargo fmt --all -- --check; then
        echo "${GREEN}✅ Code is properly formatted!${NC}"
        return 0
    else
        echo "${RED}❌ Code formatting issues found!${NC}"
        echo "${YELLOW}💡 Run 'cargo fmt' to fix formatting issues.${NC}"
        return 1
    fi
}

# Main execution
exit_code=0

# Run tests if enabled
if [ "$RUN_TESTS" = true ]; then
    if ! run_tests; then
        exit_code=1
    fi
fi

# Run clippy if enabled
if [ "$RUN_CLIPPY" = true ]; then
    if ! run_clippy; then
        exit_code=1
    fi
fi

# Run format check if enabled
if [ "$RUN_FORMAT_CHECK" = true ]; then
    if ! run_format_check; then
        exit_code=1
    fi
fi

# Final result
if [ $exit_code -eq 0 ]; then
    echo "${GREEN}🎉 All pre-commit checks passed! Proceeding with commit.${NC}"
else
    echo ""
    echo "${RED}❌ Pre-commit checks failed! Commit aborted.${NC}"
    echo "${YELLOW}💡 Fix the issues above and try committing again.${NC}"
    echo "${YELLOW}💡 To bypass this hook (not recommended): git commit --no-verify${NC}"
fi

exit $exit_code
EOF

# Make the pre-commit hook executable
chmod +x "$PRE_COMMIT_HOOK"

echo "${GREEN}✅ Pre-commit hook installed successfully!${NC}"
echo ""
echo "${BLUE}📋 What happens next:${NC}"
echo "• Tests will run automatically before each commit"
echo "• Commits will be blocked if tests fail"
echo "• You can bypass with: ${YELLOW}git commit --no-verify${NC} (not recommended)"
echo ""
echo "${BLUE}🧪 Testing the hook...${NC}"

# Test the hook by running it directly
if ./.git/hooks/pre-commit; then
    echo "${GREEN}🎉 Pre-commit hook is working correctly!${NC}"
else
    echo "${YELLOW}⚠️  Pre-commit hook test failed, but it's installed. This might be due to failing tests.${NC}"
fi

echo ""
echo "${GREEN}🎯 Setup complete! Your commits are now protected by automatic testing.${NC}"
