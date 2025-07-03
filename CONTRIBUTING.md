# Contributing to UltraFast MCP

Thank you for your interest in contributing to UltraFast MCP! This document provides guidelines and information for contributors.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Code Style](#code-style)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Release Process](#release-process)
- [Code of Conduct](#code-of-conduct)

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Git
- A GitHub account

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/mcp.git
   cd mcp
   ```
3. Add the upstream repository:
   ```bash
   git remote add upstream https://github.com/original-owner/mcp.git
   ```

## Development Setup

### Building the Project

```bash
# Build all crates
cargo build --all-targets --all-features

# Build specific crate
cargo build -p ultrafast-mcp-core

# Build examples
cargo build --examples
```

### Running Tests

```bash
# Run all tests
cargo test --all-targets --all-features

# Run tests for specific crate
cargo test -p ultrafast-mcp-core

# Run integration tests
cargo test --test integration_tests
cargo test --test http_integration_tests

# Run with verbose output
cargo test -- --nocapture
```

### Code Quality Checks

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Check documentation
cargo doc --no-deps --all-features

# Security audit
cargo audit
```

## Code Style

### Rust Style Guidelines

- Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/style/naming/README.html)
- Use `cargo fmt` to format code
- Use `cargo clippy` to check for common issues
- Write meaningful commit messages

### Documentation

- Document all public APIs
- Use doc comments (`///`) for public items
- Include examples in documentation
- Update README.md when adding new features

### Commit Messages

Use conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Test changes
- `chore`: Build/tooling changes

Examples:
```
feat(core): add new transport protocol support
fix(auth): resolve OAuth token refresh issue
docs(readme): update installation instructions
```

## Testing

### Writing Tests

- Write unit tests for all new functionality
- Include integration tests for complex features
- Test error conditions and edge cases
- Use descriptive test names

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = "test";
        
        // Act
        let result = function(input);
        
        // Assert
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_async_function() {
        // Async test implementation
    }
}
```

### Running Specific Tests

```bash
# Run specific test
cargo test test_name

# Run tests matching pattern
cargo test pattern

# Run tests in specific module
cargo test module_name::tests
```

## Pull Request Process

### Before Submitting

1. **Update your fork**: Keep your fork up to date with the main repository
   ```bash
   git fetch upstream
   git checkout main
   git merge upstream/main
   ```

2. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes**:
   - Write your code
   - Add tests
   - Update documentation
   - Run all checks locally

4. **Commit your changes**:
   ```bash
   git add .
   git commit -m "feat(scope): description"
   ```

5. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

### Submitting a PR

1. Go to your fork on GitHub
2. Click "New Pull Request"
3. Select your feature branch
4. Fill out the PR template
5. Submit the PR

### PR Review Process

1. **Automated Checks**: The PR will run through automated checks:
   - Code formatting
   - Clippy linting
   - Tests
   - Security audit
   - Cross-platform testing
   - Code coverage

2. **Review**: Maintainers will review your code for:
   - Correctness
   - Performance
   - Security
   - Documentation
   - Test coverage

3. **Feedback**: You may receive feedback requesting changes

4. **Approval**: Once approved, your PR will be merged

### PR Guidelines

- **Keep PRs focused**: One feature or fix per PR
- **Write clear descriptions**: Explain what and why, not how
- **Include tests**: All new code should have tests
- **Update documentation**: Keep docs in sync with code
- **Respond to feedback**: Address review comments promptly

## Release Process

### Versioning

We follow [Semantic Versioning](https://semver.org/):
- `MAJOR.MINOR.PATCH`
- Major: Breaking changes
- Minor: New features (backward compatible)
- Patch: Bug fixes (backward compatible)

### Release Checklist

Before a release:

- [ ] All tests pass
- [ ] Documentation is up to date
- [ ] CHANGELOG.md is updated
- [ ] Version numbers are updated
- [ ] Security audit passes
- [ ] Performance benchmarks are acceptable

### Creating a Release

1. Create a release branch
2. Update version numbers
3. Update CHANGELOG.md
4. Create a PR
5. After approval, tag the release
6. Publish to crates.io

## Code of Conduct

### Our Standards

- Be respectful and inclusive
- Use welcoming and inclusive language
- Be collaborative
- Focus on what is best for the community
- Show empathy towards other community members

### Enforcement

- Unacceptable behavior will not be tolerated
- Violations will be addressed promptly
- Maintainers have the right to remove, edit, or reject contributions

## Getting Help

- **Issues**: Use GitHub Issues for bug reports and feature requests
- **Discussions**: Use GitHub Discussions for questions and general discussion
- **Documentation**: Check the docs/ directory for detailed documentation

## License

By contributing to UltraFast MCP, you agree that your contributions will be licensed under the same license as the project (Apache 2.0 + MIT).

Thank you for contributing to UltraFast MCP! 🚀 