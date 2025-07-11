# Changelog

All notable changes to the UltraFast MCP project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive GitHub Actions CI/CD pipeline
- Cross-platform testing (Ubuntu, Windows, macOS)
- Automated security auditing with cargo-audit
- Performance benchmarking and regression testing
- Automated documentation generation and deployment
- Pull request validation workflows
- Nightly maintenance workflows

### Changed
- Fixed 50+ compiler warnings across all crates
- Removed unused imports and dead code
- Improved code quality and maintainability
- Enhanced error handling and validation

### Fixed
- Unused imports in auth, transport, and server crates
- Unused variables in test files
- Dead code warnings in server implementation
- Missing documentation comments

## [0.1.0] - 2024-12-19

### Added
- **Initial Release**: High-performance MCP implementation in Rust
- **Core Protocol**: 100% MCP 2025-06-18 specification compliance
- **Transport Layer**: 
  - Streamable HTTP transport with session management
  - stdio transport for local tools
  - Cross-platform support (Linux, Windows, macOS)
- **Authentication**: OAuth 2.1 with PKCE and dynamic client registration
- **Server Implementation**: 
  - Ergonomic builder pattern API
  - Type-safe tool and resource handlers
  - Comprehensive error handling
- **Client Implementation**:
  - Easy-to-use client API
  - Automatic reconnection and retry logic
  - Progress tracking and cancellation support
- **CLI Tool**: Complete command-line interface with project scaffolding
- **Monitoring**: OpenTelemetry integration for observability
- **Documentation**: Comprehensive API documentation and examples

### Features
- **Tools**: Function execution with JSON Schema validation
- **Resources**: URI-based resource management with templates
- **Prompts**: Template-based prompt system with arguments
- **Sampling**: Server-initiated LLM completions
- **Roots**: Filesystem boundary management
- **Elicitation**: User input collection
- **Logging**: RFC 5424 compliant structured logging
- **Completion**: Argument autocompletion system

### Architecture
- **ultrafast-mcp**: Main crate with unified APIs
- **ultrafast-mcp-core**: Core protocol implementation
- **ultrafast-mcp-server**: Server-side implementation
- **ultrafast-mcp-client**: Client-side implementation
- **ultrafast-mcp-transport**: Transport layer (stdio/HTTP)
- **ultrafast-mcp-auth**: OAuth 2.1 authentication
- **ultrafast-mcp-cli**: Command-line interface
- **ultrafast-mcp-monitoring**: Observability and metrics

### Examples
- **Basic Echo**: Simple server-client communication
- **File Operations**: File system manipulation tools
- **HTTP Server**: Web-based MCP server with authentication
- **Advanced Features**: Complex tool and resource implementations

### Performance
- **Zero-copy optimizations** for high-throughput scenarios
- **Async-first design** with tokio integration
- **Memory safety** guaranteed by Rust
- **10x performance improvement** over HTTP+SSE transport

### Security
- **OAuth 2.1 compliance** with PKCE support
- **Secure session management** with automatic cleanup
- **Input validation** with JSON Schema
- **Rate limiting** and connection pooling

### Developer Experience
- **Ergonomic APIs** inspired by FastMCP
- **Type-safe** with automatic schema generation
- **Comprehensive CLI** with project scaffolding
- **5 working examples** with full documentation
- **Extensive test coverage** with 81+ tests

---

## Versioning

This project follows [Semantic Versioning](https://semver.org/):

- **MAJOR** version for incompatible API changes
- **MINOR** version for added functionality in a backwards compatible manner
- **PATCH** version for backwards compatible bug fixes

## Migration Guide

### From Pre-0.1.0
This is the initial release, so no migration is required.

### Breaking Changes
- None in this release

### Deprecations
- SSE transport is deprecated in favor of StreamableHttpTransport (MCP 2025-03-26 specification)

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.