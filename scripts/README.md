# ULTRAFAST_MCP Version Management Scripts

This directory contains scripts to help manage versions and prevent CI rate limiting issues.

## Scripts

### `version-check.sh`

Checks if a version already exists on crates.io before publishing.

**Usage:**
```bash
./scripts/version-check.sh
```

**What it does:**
- Gets the current version from `Cargo.toml`
- Checks all crates against crates.io to see if the version already exists
- Exits with error if any version already exists
- Provides colored output for easy reading

**Example output:**
```
[INFO] Checking version availability for ULTRAFAST_MCP...
[INFO] Current version: 202506018.1.0-rc.1.5
[INFO] Checking ultrafast-mcp-core@202506018.1.0-rc.1.5...
[INFO] ✓ ultrafast-mcp-core@202506018.1.0-rc.1.5 is available
[INFO] All versions are available for publishing!
```

### `bump-version.sh`

Automatically bumps versions and updates all dependencies.

**Usage:**
```bash
# Bump patch version (1.0.0 -> 1.0.1)
./scripts/bump-version.sh patch

# Bump minor version (1.0.0 -> 1.1.0)
./scripts/bump-version.sh minor

# Bump major version (1.0.0 -> 2.0.0)
./scripts/bump-version.sh major

# Bump release candidate (1.0.0-rc.1 -> 1.0.0-rc.2)
./scripts/bump-version.sh rc

# Dry run to see what would change
./scripts/bump-version.sh --dry-run patch
```

**What it does:**
- Validates version format (semver)
- Updates workspace version in `Cargo.toml`
- Updates all internal crate dependencies
- Verifies all changes are consistent
- Provides helpful next steps

**Example output:**
```
[HEADER] Current version: 202506018.1.0-rc.1.4
[HEADER] New version: 202506018.1.0-rc.1.5
[INFO] Updating workspace version to 202506018.1.0-rc.1.5...
[INFO] ✓ Updated workspace version
[INFO] Updating crate dependencies...
[INFO] ✓ Updated crates/ultrafast-mcp-core/Cargo.toml
[INFO] ✓ Version bumped successfully to 202506018.1.0-rc.1.5
[INFO] Don't forget to:
[INFO]   1. Commit the changes: git add -A && git commit -m "Bump version to 202506018.1.0-rc.1.5"
[INFO]   2. Create a release tag: git tag v202506018.1.0-rc.1.5
[INFO]   3. Push changes: git push && git push --tags
```

## CI/CD Optimizations

### Rate Limiting Prevention

The CI/CD pipeline has been optimized to prevent rate limiting issues:

1. **Version Pre-check**: Before publishing, check if versions already exist
2. **Staggered Publishing**: Use matrix strategy with delays between crates
3. **Retry Logic**: Exponential backoff for failed publishes
4. **Dry Run Validation**: Test package creation before actual publishing

### Pre-Release Workflow

Use the GitHub Actions "Pre-Release Validation" workflow to validate versions before release:

1. Go to Actions → Pre-Release Validation
2. Click "Run workflow"
3. Enter the version to validate
4. Choose dry run mode
5. Review the validation results

### Publishing Strategy

The optimized publishing strategy:

1. **Core dependencies first**: `ultrafast-mcp-core` (0s delay)
2. **Auth and transport**: `ultrafast-mcp-auth` (2min delay)
3. **Transport layer**: `ultrafast-mcp-transport` (4min delay)
4. **Monitoring**: `ultrafast-mcp-monitoring` (6min delay)
5. **Server**: `ultrafast-mcp-server` (8min delay)
6. **Client**: `ultrafast-mcp-client` (10min delay)
8. **CLI**: `ultrafast-mcp-cli` (14min delay)
9. **Main crate**: `ultrafast-mcp` (16min delay)

Total publishing time: ~16 minutes with proper rate limiting protection.

## Best Practices

1. **Always run pre-release validation** before creating a release
2. **Use the bump script** instead of manually editing versions
3. **Check version availability** before triggering CI
4. **Monitor CI logs** for rate limiting warnings
5. **Use dry run mode** for testing

## Troubleshooting

### Rate Limiting Issues

If you encounter rate limiting:

1. **Wait for the specified time** in the error message
2. **Email help@crates.io** to request a limit increase
3. **Use a different account** if available
4. **Increase delays** in the CI workflow if needed

### Version Conflicts

If versions already exist:

1. **Run the bump script** to create a new version
2. **Use the version check script** to verify availability
3. **Update the release tag** to match the new version

### CI Failures

If CI fails:

1. **Check the pre-release validation** workflow first
2. **Review the error logs** for specific issues
3. **Fix the issues** and re-run validation
4. **Only create releases** when all validation passes 