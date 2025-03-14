# Contributing to RustCC

Thank you for your interest in contributing to RustCC! This document provides guidelines and instructions for contributing to this project.

## Code of Conduct

This project and everyone participating in it is governed by our Code of Conduct. By participating, you are expected to uphold this code.

## How Can I Contribute?

### Reporting Bugs

This section guides you through submitting a bug report for RustCC. Following these guidelines helps maintainers understand your report, reproduce the behavior, and find related reports.

- **Use a clear and descriptive title** for the issue to identify the problem.
- **Describe the exact steps which reproduce the problem** in as many details as possible.
- **Provide specific examples** to demonstrate the steps.
- **Describe the behavior you observed after following the steps** and point out what exactly is the problem with that behavior.
- **Explain which behavior you expected to see instead and why.**
- **Include screenshots or animated GIFs** which show you following the described steps and clearly demonstrate the problem.
- **If the problem is related to performance or memory**, include a CPU profile capture and a memory heap snapshot with your report.
- **If the problem wasn't triggered by a specific action**, describe what you were doing before the problem happened.

### Suggesting Enhancements

This section guides you through submitting an enhancement suggestion for RustCC, including completely new features and minor improvements to existing functionality.

- **Use a clear and descriptive title** for the issue to identify the suggestion.
- **Provide a step-by-step description of the suggested enhancement** in as many details as possible.
- **Provide specific examples to demonstrate the steps** or point to similar features or implementations in other projects.
- **Describe the current behavior** and **explain which behavior you expected to see instead** and why.
- **Explain why this enhancement would be useful** to most RustCC users.
- **Specify which version of RustCC you're using.**

### Pull Requests

- Fill in the required template
- Follow the Rust style guide
- Include appropriate test cases
- End all files with a newline
- Update the README.md with details of changes to the interface

## Development Workflow

### Setting Up Your Development Environment

1. Fork the repository
2. Clone your fork:
```
git clone https://github.com/your-username/rustcc.git
```
3. Add the upstream repository:
```
git remote add upstream https://github.com/TheBitty/rustcc.git
```
4. Create a branch for your work:
```
git checkout -b feature/your-feature-name
```

### Running Tests

Before submitting a PR, make sure all tests pass:

```
cargo test
cargo test --features llvm-backend  # If you have LLVM installed
```

### Code Style

We use rustfmt and clippy to maintain code quality. Please run the following before submitting your PR:

```
cargo fmt
cargo clippy
```

## Rust Coding Standards

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Write documentation for all public APIs
- Use meaningful variable and function names
- Avoid unsafe code unless absolutely necessary
- Properly handle errors (avoid unwrap/expect in production code)
- Write tests for new functionality

## Git Commit Guidelines

- Use concise but descriptive commit messages
- Reference issues and pull requests where appropriate
- Commit small, logical changes frequently
- Rebase your branch before submitting a PR

## License

By contributing to RustCC, you agree that your contributions will be licensed under the project's MIT license. 