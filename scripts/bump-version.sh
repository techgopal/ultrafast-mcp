#!/bin/bash

# Version bump script for ULTRAFAST_MCP
# This script helps bump versions and update all dependencies

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

print_header() {
    echo -e "${BLUE}[HEADER]${NC} $1"
}

# Function to validate version format
validate_version() {
    local version=$1
    
    # Check if version matches semver format
    if [[ $version =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?(\+[a-zA-Z0-9.-]+)?$ ]]; then
        return 0
    else
        return 1
    fi
}

# Function to bump version
bump_version() {
    local current_version=$1
    local bump_type=$2
    
    # Parse current version
    local major=$(echo $current_version | cut -d. -f1)
    local minor=$(echo $current_version | cut -d. -f2)
    local patch=$(echo $current_version | cut -d. -f3 | cut -d- -f1)
    local prerelease=$(echo $current_version | cut -d- -f2-)
    
    case $bump_type in
        "major")
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        "minor")
            minor=$((minor + 1))
            patch=0
            ;;
        "patch")
            patch=$((patch + 1))
            ;;
        "rc")
            # For release candidates, increment the rc number
            if [[ $prerelease =~ ^rc\.([0-9]+)$ ]]; then
                local rc_num=${BASH_REMATCH[1]}
                rc_num=$((rc_num + 1))
                prerelease="rc.$rc_num"
            else
                # If no rc exists, create rc.1
                prerelease="rc.1"
            fi
            ;;
        *)
            print_error "Invalid bump type: $bump_type. Use: major, minor, patch, or rc"
            exit 1
            ;;
    esac
    
    # Construct new version
    if [ -n "$prerelease" ]; then
        echo "$major.$minor.$patch-$prerelease"
    else
        echo "$major.$minor.$patch"
    fi
}

# Function to update workspace version
update_workspace_version() {
    local new_version=$1
    
    print_status "Updating workspace version to $new_version..."
    
    # Update main Cargo.toml
    sed -i.bak "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
    rm Cargo.toml.bak
    
    print_status "✓ Updated workspace version"
}

# Function to update crate dependencies
update_crate_dependencies() {
    local new_version=$1
    
    print_status "Updating crate dependencies..."
    
    # Find all Cargo.toml files in crates directory
    for cargo_file in crates/*/Cargo.toml; do
        if [ -f "$cargo_file" ]; then
            print_status "Updating dependencies in $cargo_file..."
            
            # Update all ultrafast-mcp dependencies
            sed -i.bak "s/ultrafast-mcp-[a-zA-Z-]* = { path = \"[^\"]*\", version = \"[^\"]*\"/&/g" "$cargo_file"
            sed -i.bak "s/version = \"[^\"]*\"/version = \"$new_version\"/g" "$cargo_file"
            rm "$cargo_file.bak"
            
            print_status "✓ Updated $cargo_file"
        fi
    done
}

# Function to verify changes
verify_changes() {
    local new_version=$1
    
    print_status "Verifying version changes..."
    
    # Check workspace version
    local workspace_version=$(grep '^version = ' Cargo.toml | cut -d'"' -f2)
    if [ "$workspace_version" != "$new_version" ]; then
        print_error "Workspace version mismatch: expected $new_version, got $workspace_version"
        exit 1
    fi
    
    # Check all crate dependencies
    for cargo_file in crates/*/Cargo.toml; do
        if [ -f "$cargo_file" ]; then
            local crate_name=$(basename $(dirname "$cargo_file"))
            print_status "Verifying $crate_name dependencies..."
            
            # Check if all ultrafast-mcp dependencies use the new version
            if grep -q "ultrafast-mcp" "$cargo_file"; then
                if ! grep -q "version = \"$new_version\"" "$cargo_file"; then
                    print_error "Dependency version mismatch in $cargo_file"
                    exit 1
                fi
            fi
        fi
    done
    
    print_status "✓ All version changes verified"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS] <bump_type>"
    echo ""
    echo "Bump types:"
    echo "  major    - Bump major version (1.0.0 -> 2.0.0)"
    echo "  minor    - Bump minor version (1.0.0 -> 1.1.0)"
    echo "  patch    - Bump patch version (1.0.0 -> 1.0.1)"
    echo "  rc       - Bump release candidate (1.0.0-rc.1 -> 1.0.0-rc.2)"
    echo ""
    echo "Options:"
    echo "  --dry-run    - Show what would be changed without making changes"
    echo "  --help       - Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 patch                    # Bump patch version"
    echo "  $0 --dry-run minor          # Show what minor bump would do"
    echo "  $0 rc                       # Bump release candidate"
}

# Main function
main() {
    local dry_run=false
    local bump_type=""
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                dry_run=true
                shift
                ;;
            --help)
                show_usage
                exit 0
                ;;
            -*)
                print_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
            *)
                if [ -z "$bump_type" ]; then
                    bump_type=$1
                else
                    print_error "Multiple bump types specified"
                    exit 1
                fi
                shift
                ;;
        esac
    done
    
    if [ -z "$bump_type" ]; then
        print_error "No bump type specified"
        show_usage
        exit 1
    fi
    
    # Get current version
    local current_version=$(grep '^version = ' Cargo.toml | cut -d'"' -f2)
    print_header "Current version: $current_version"
    
    # Calculate new version
    local new_version=$(bump_version "$current_version" "$bump_type")
    print_header "New version: $new_version"
    
    # Validate new version
    if ! validate_version "$new_version"; then
        print_error "Invalid version format: $new_version"
        exit 1
    fi
    
    if [ "$dry_run" = true ]; then
        print_warning "DRY RUN - No changes will be made"
        print_status "Would update workspace version to: $new_version"
        print_status "Would update all crate dependencies to: $new_version"
    else
        # Update versions
        update_workspace_version "$new_version"
        update_crate_dependencies "$new_version"
        verify_changes "$new_version"
        
        print_status "✓ Version bumped successfully to $new_version"
        print_status "Don't forget to:"
        print_status "  1. Commit the changes: git add -A && git commit -m \"Bump version to $new_version\""
        print_status "  2. Create a release tag: git tag v$new_version"
        print_status "  3. Push changes: git push && git push --tags"
    fi
}

# Run main function
main "$@" 