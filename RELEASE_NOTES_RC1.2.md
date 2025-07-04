# UltraFast MCP v202506018.1.0-rc.1.2

## 🚀 Release Candidate 1.2 for 202506018.1.0

This is the second release candidate for the 202506018.1.0 series of UltraFast MCP. This release includes bug fixes, documentation improvements, and enhanced CI/CD workflows.

---

## 🆕 What's New in RC1.2

- **Version bump:** All crates updated to `202506018.1.0-rc.1.2`
- **CI/CD improvements:** Fixed deprecated GitHub Actions versions
- **Documentation:** Comprehensive README rewrite with code examples
- **Version consistency:** Enhanced workspace version inheritance validation
- **Release candidate status:** Clear pre-production warnings

---

## 🛠️ Notable Changes

### CI/CD Improvements
- ✅ Updated `actions/upload-artifact@v3` → `actions/upload-artifact@v4`
- ✅ Updated `actions/cache@v3` → `actions/cache@v4`
- ✅ Fixed version consistency check for workspace inheritance
- ✅ Enhanced release workflow validation

### Documentation Enhancements
- ✅ Complete README rewrite for release candidate status
- ✅ Added comprehensive code examples for all features
- ✅ Included advanced features documentation
- ✅ Added file operations, HTTP authentication, and sampling examples
- ✅ Updated branding consistency (UltraFast MCP)
- ✅ Fixed repository URLs and version references

### Bug Fixes
- ✅ Resolved version inheritance validation in CI
- ✅ Fixed workspace package version consistency
- ✅ Improved error handling in release workflows

---

## 📦 Crates Published

- `ultrafast-mcp-core` v202506018.1.0-rc.1.2
- `ultrafast-mcp-transport` v202506018.1.0-rc.1.2
- `ultrafast-mcp-auth` v202506018.1.0-rc.1.2
- `ultrafast-mcp-monitoring` v202506018.1.0-rc.1.2
- `ultrafast-mcp-server` v202506018.1.0-rc.1.2
- `ultrafast-mcp-client` v202506018.1.0-rc.1.2
- `ultrafast-mcp-macros` v202506018.1.0-rc.1.2
- `ultrafast-mcp-cli` v202506018.1.0-rc.1.2
- `ultrafast-mcp` v202506018.1.0-rc.1.2 (main crate)

---

## 📝 How to Upgrade

Update your dependencies in `Cargo.toml` to use `=202506018.1.0-rc.1.2` for the relevant crates:

```toml
[dependencies]
ultrafast-mcp = { version = "202506018.1.0-rc.1.2", features = ["http", "oauth"] }
```

---

## 🔧 Code Examples

### Basic Server
```rust
use ultrafast_mcp::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let server = UltraFastServer::new("My MCP Server")
        .with_protocol_version("2025-06-18");
    
    server.tool("greet", |name: String, ctx: Context| async move {
        Ok(format!("Hello, {}! Welcome to UltraFast MCP!", name))
    })
    .description("Greet a user by name");
    
    server.run_stdio().await?;
    Ok(())
}
```

### HTTP Server with OAuth
```rust
use ultrafast_mcp::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let server = UltraFastServer::new("HTTP MCP Server")
        .with_capabilities(ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: true }),
            ..Default::default()
        });
    
    server.tool("get_user_info", |_: (), ctx: Context| async move {
        let token = ctx.get_auth_token()?;
        // Make authenticated request...
        Ok(serde_json::json!({"user": "authenticated"}))
    })
    .requires_auth(true);
    
    server.run_http("127.0.0.1:8080", None).await?;
    Ok(())
}
```

---

## 🚨 Important Notes

### Release Candidate Status
This is **Release Candidate 1.2** and should be considered **pre-production** software. We recommend:
- Thorough testing in your environment
- Validation of all features you plan to use
- Testing with your specific use cases
- Reporting any issues found

### Breaking Changes
No breaking changes from RC1. This is a patch release with bug fixes and documentation improvements.

---

## 🙏 Thanks

Thank you to all contributors and testers for making this release possible!

---

## 📞 Support

- **Documentation**: [https://docs.rs/ultrafast-mcp](https://docs.rs/ultrafast-mcp)
- **Issues**: [GitHub Issues](https://github.com/techgopal/ultrafast-mcp/issues)
- **Discussions**: [GitHub Discussions](https://github.com/techgopal/ultrafast-mcp/discussions) 