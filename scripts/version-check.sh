#!/bin/bash

# Version check script for ULTRAFAST_MCP
# This script checks if a version already exists on crates.io before publishing

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Get current version from workspace
get_workspace_version() {
    grep '^version = ' Cargo.toml | cut -d'"' -f2
}

# Check if version exists for a specific crate
check_crate_version() {
    local crate_name=$1
    local version=$2
    
    # Use cargo search to check if version exists
    if cargo search "$crate_name" --limit 10 | grep -q "$version"; then
        return 0  # Version exists
    else
        return 1  # Version doesn't exist
    fi
}

# Main function
main() {
    print_status "Checking version availability for ULTRAFAST_MCP..."
    
    local version=$(get_workspace_version)
    print_status "Current version: $version"
    
    # List of crates to check
    local crates=(
        "ultrafast-mcp-core"
        "ultrafast-mcp-auth"
        "ultrafast-mcp-transport"
        "ultrafast-mcp-monitoring"
        "ultrafast-mcp-server"
        "ultrafast-mcp-client"
        "ultrafast-mcp-cli"
        "ultrafast-mcp"
    )
    
    local versions_exist=false
    
    # Check each crate
    for crate in "${crates[@]}"; do
        print_status "Checking $crate@$version..."
        
        if check_crate_version "$crate" "$version"; then
            print_error "Version $version already exists for $crate"
            versions_exist=true
        else
            print_status "✓ $crate@$version is available"
        fi
    done
    
    if [ "$versions_exist" = true ]; then
        print_error "Some versions already exist on crates.io. Please bump the version before publishing."
        exit 1
    else
        print_status "All versions are available for publishing!"
        exit 0
    fi
}

# Run main function
main "$@" 