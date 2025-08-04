# Git Pre-commit Hook Configuration for rs-mock-server

This repository includes a Git pre-commit hook that automatically runs tests before allowing commits.

## What it does

The pre-commit hook:

-   Runs `cargo test --quiet` before each commit
-   Prevents the commit if any tests fail
-   Shows clear success/failure messages
-   Provides guidance on how to fix issues

## How to use

The hook is automatically active in this repository. When you run:

```bash
git commit -m "Your commit message"
```

The hook will:

1. Run all tests
2. If tests pass: Allow the commit to proceed
3. If tests fail: Abort the commit and show an error message

## Bypassing the hook (not recommended)

If you absolutely need to commit without running tests (e.g., work-in-progress commits), you can bypass the hook with:

```bash
git commit --no-verify -m "WIP: commit message"
```

**Note:** This is not recommended for main branch commits.

## Customizing the hook

The pre-commit hook is located at `.git/hooks/pre-commit`. You can modify it to:

-   Run specific test suites
-   Add additional checks (linting, formatting, etc.)
-   Change the test timeout
-   Add different behavior for different branches

## Alternative: Using a dedicated tool

For more advanced pre-commit functionality, consider using tools like:

-   [pre-commit](https://pre-commit.com/)
-   [husky](https://github.com/typicode/husky) (for Node.js projects)
-   [lefthook](https://github.com/evilmartians/lefthook)

These tools provide more sophisticated configuration and multi-language support.
