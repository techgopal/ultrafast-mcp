# UltraFast MCP v202506018.1.0-rc.1.3

## 🚀 Release Candidate 1.3 for 202506018.1.0

This is the third release candidate for the 202506018.1.0 series of UltraFast MCP. This release includes packaging fixes, dependency version management, and improved CI/CD workflows.

---

## 🆕 What's New in RC1.3

- **Version bump:** All crates updated to `202506018.1.0-rc.1.3`
- **Packaging fixes:** Added version specifications to all internal dependencies
- **README compliance:** Each crate now has its own README.md for crates.io
- **CI/CD improvements:** Enhanced release workflow validation
- **Dependency management:** Proper version inheritance across workspace

---

## 🛠️ Notable Changes

### Packaging & Dependencies
- ✅ Added `version = "202506018.1.0-rc.1.3"` to all internal path dependencies
- ✅ Created individual README.md files for each crate
- ✅ Set `readme = "README.md"` in all crate Cargo.toml files
- ✅ Fixed workspace version inheritance for packaging compliance

### CI/CD Improvements
- ✅ Enhanced version consistency validation
- ✅ Improved package validation workflow
- ✅ Fixed GitHub Actions deprecation warnings
- ✅ Streamlined release process

### Documentation
- ✅ Updated README.md with correct feature flags
- ✅ Fixed repository URLs and branding consistency
- ✅ Added comprehensive code examples
- ✅ Clarified release candidate status

---

## 📦 Crates Published

- `ultrafast-mcp-core` v202506018.1.0-rc.1.3
- `ultrafast-mcp-transport` v202506018.1.0-rc.1.3
- `ultrafast-mcp-auth` v202506018.1.0-rc.1.3
- `ultrafast-mcp-monitoring` v202506018.1.0-rc.1.3
- `ultrafast-mcp-server` v202506018.1.0-rc.1.3
- `ultrafast-mcp-client` v202506018.1.0-rc.1.3
- `ultrafast-mcp-macros` v202506018.1.0-rc.1.3
- `ultrafast-mcp-cli` v202506018.1.0-rc.1.3
- `ultrafast-mcp` v202506018.1.0-rc.1.3 (main crate)

---

## 📝 How to Upgrade

Update your dependencies in `Cargo.toml` to use `=202506018.1.0-rc.1.3` for the relevant crates:

```toml
[dependencies]
ultrafast-mcp = { version = "202506018.1.0-rc.1.3", features = ["http", "oauth"] }
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
This is **Release Candidate 1.3** and should be considered **pre-production** software. We recommend:
- Thorough testing in your environment
- Validation of all features you plan to use
- Testing with your specific use cases
- Reporting any issues found

### Breaking Changes
No breaking changes from RC1.2. This is a patch release with packaging and dependency fixes.

### Packaging Compliance
All crates now have:
- Proper version specifications for internal dependencies
- Individual README.md files
- Correct readme field configuration
- Full crates.io packaging compliance

---

## 🙏 Thanks

Thank you to all contributors and testers for making this release possible!

---

## 📞 Support

- **Documentation**: [https://docs.rs/ultrafast-mcp](https://docs.rs/ultrafast-mcp)
- **Issues**: [GitHub Issues](https://github.com/techgopal/ultrafast-mcp/issues)
- **Discussions**: [GitHub Discussions](https://github.com/techgopal/ultrafast-mcp/discussions) 