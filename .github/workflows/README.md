# GitHub Actions Workflows

This directory contains GitHub Actions workflows for the UltraFast MCP project.

## Workflows Overview

### 1. **ci.yml** - Comprehensive CI/CD Pipeline
**Triggers:** Push to main/develop, Pull requests, Releases

**Jobs:**
- **Test Matrix**: Tests on Ubuntu, Windows, macOS with stable and beta Rust
- **Minimal Features**: Tests with different feature combinations
- **Documentation**: Builds and uploads documentation
- **Security Audit**: Runs cargo-audit for vulnerability checks
- **Benchmarks**: Performance testing (main branch only)
- **Package Validation**: Validates all crates can be packaged
- **Publish**: Publishes to crates.io (release only)
- **Deploy Docs**: Deploys documentation to GitHub Pages

### 2. **pr.yml** - Pull Request Checks
**Triggers:** Pull requests to main/develop, push to PR branches

**Jobs:**
- **PR Checks**: Fast formatting, clippy, tests, docs (runs on every PR)
- **Security Audit**: Vulnerability scanning (runs on every PR)
- **Cross-Platform**: Tests on Windows and macOS (non-draft PRs only)
- **Feature Testing**: Tests with different feature combinations (non-draft PRs only)
- **Code Coverage**: Coverage analysis and reporting (non-draft PRs only)
- **Integration Tests**: End-to-end testing (non-draft PRs only)
- **Performance Check**: Benchmark validation (non-draft PRs only)

### 3. **release.yml** - Release Pipeline
**Triggers:** Release published

**Jobs:**
- **Validate**: Version consistency, full test suite, security audit
- **Publish**: Publishes all 9 crates in dependency order
- **Deploy Docs**: Deploys documentation to GitHub Pages
- **Verify Installation**: Tests installation and basic functionality

### 4. **nightly.yml** - Nightly Maintenance
**Triggers:** Daily at 2 AM UTC, manual dispatch

**Jobs:**
- **Nightly Build**: Tests with nightly Rust toolchain
- **Update Dependencies**: Checks for outdated dependencies
- **Security Audit**: Daily security checks
- **Performance**: Performance regression testing
- **Documentation**: Nightly documentation generation
- **Code Quality**: Metrics and quality checks

## Setup Requirements

### Required Secrets

1. **`CARGO_REGISTRY_TOKEN`**: Your crates.io API token
   - Get from: https://crates.io/settings/tokens
   - Required for publishing to crates.io

2. **`GITHUB_TOKEN`**: Automatically provided by GitHub
   - Used for GitHub Pages deployment

### Optional Secrets

3. **`RUSTUP_DIST_ROOT`**: Custom Rust distribution (if needed)
4. **`RUSTUP_UPDATE_ROOT`**: Custom Rust update server (if needed)

## Workflow Features

### 🔄 Dependency Management
- Automatic caching of dependencies using `rust-cache`
- Cross-platform dependency installation
- SSL development libraries for Ubuntu

### 🛡️ Security
- Automated security audits with `cargo-audit`
- Vulnerability scanning and reporting
- Dependency update monitoring

### 📊 Performance
- Benchmark execution and result storage
- Performance regression detection
- Baseline comparison

### 📚 Documentation
- Automatic documentation generation
- GitHub Pages deployment
- Documentation validation

### 🚀 Publishing
- Sequential crate publishing in dependency order
- Proper delays between publishes
- Installation verification
- Cross-platform testing

## Usage

### For Contributors
1. **Pull Requests**: Automatically tested via `pr.yml`
2. **Code Quality**: Formatting and clippy checks enforced
3. **Security**: Automatic vulnerability scanning

### For Maintainers
1. **Releases**: Create a GitHub release to trigger publishing
2. **Nightly Monitoring**: Check nightly workflow results
3. **Performance Tracking**: Monitor benchmark results

### Manual Triggers
- **Nightly Workflow**: Can be triggered manually via GitHub UI
- **Release Workflow**: Triggered by creating a GitHub release

## Troubleshooting

### Common Issues

1. **Publishing Fails**
   - Check `CARGO_REGISTRY_TOKEN` is set correctly
   - Verify crate names are available on crates.io
   - Ensure version numbers are consistent

2. **Tests Fail on Windows/macOS**
   - Check for platform-specific dependencies
   - Verify SSL libraries are available
   - Check for path separator issues

3. **Documentation Build Fails**
   - Check for missing documentation comments
   - Verify all public APIs are documented
   - Check for broken links in documentation

### Performance Optimization

1. **Cache Dependencies**: Uses `rust-cache` for faster builds
2. **Parallel Jobs**: Multiple jobs run in parallel where possible
3. **Selective Testing**: Different test strategies for different triggers

## Customization

### Adding New Jobs
1. Add job definition to appropriate workflow file
2. Configure triggers and dependencies
3. Add required secrets if needed

### Modifying Test Matrix
1. Edit the `strategy.matrix` section in `ci.yml`
2. Add new operating systems or Rust versions
3. Update dependency installation steps if needed

### Adding New Platforms
1. Add platform to test matrix
2. Configure platform-specific dependencies
3. Test thoroughly before merging

## Monitoring

### Workflow Status
- Check GitHub Actions tab for current status
- Monitor for failed workflows
- Review performance trends

### Metrics
- Test execution time
- Build success rates
- Security audit results
- Performance benchmark trends

## Best Practices

1. **Keep Workflows Fast**: Use caching and parallel execution
2. **Fail Fast**: Run quick checks first, expensive tests later
3. **Security First**: Always run security audits
4. **Document Changes**: Update this README when modifying workflows
5. **Test Locally**: Use `act` to test workflows locally before pushing 