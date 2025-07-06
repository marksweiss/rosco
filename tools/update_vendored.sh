#!/bin/bash
# Update vendored tools script
# This script updates all git submodules in the tools directory

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if a directory is a git submodule
is_submodule() {
    local dir="$1"
    if [ -d "$dir" ] && [ -f "$dir/.git" ]; then
        return 0
    else
        return 1
    fi
}

# Function to update a single submodule
update_submodule() {
    local submodule_path="$1"
    local submodule_name=$(basename "$submodule_path")
    
    print_status "Updating submodule: $submodule_name"
    
    if [ ! -d "$submodule_path" ]; then
        print_warning "Submodule directory $submodule_path does not exist"
        return 1
    fi
    
    cd "$submodule_path"
    
    # Check if this is actually a submodule
    if ! is_submodule "$submodule_path"; then
        print_warning "$submodule_name is not a git submodule, skipping"
        cd "$PROJECT_ROOT"
        return 1
    fi
    
    # Get current commit
    local current_commit=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
    
    # Fetch latest changes
    if git fetch origin > /dev/null 2>&1; then
        # Check if there are updates
        local latest_commit=$(git rev-parse origin/main 2>/dev/null || git rev-parse origin/master 2>/dev/null || echo "unknown")
        
        if [ "$current_commit" != "$latest_commit" ]; then
            print_status "New commits available for $submodule_name"
            
            # Update to latest
            if git checkout main > /dev/null 2>&1 || git checkout master > /dev/null 2>&1; then
                if git pull origin main > /dev/null 2>&1 || git pull origin master > /dev/null 2>&1; then
                    print_success "Updated $submodule_name to latest"
                else
                    print_error "Failed to pull latest changes for $submodule_name"
                    cd "$PROJECT_ROOT"
                    return 1
                fi
            else
                print_error "Failed to checkout main/master for $submodule_name"
                cd "$PROJECT_ROOT"
                return 1
            fi
        else
            print_success "$submodule_name is already up to date"
        fi
    else
        print_error "Failed to fetch updates for $submodule_name"
        cd "$PROJECT_ROOT"
        return 1
    fi
    
    cd "$PROJECT_ROOT"
}

# Function to initialize submodules if needed
init_submodules() {
    print_status "Initializing submodules..."
    
    if git submodule status | grep -q "^-"; then
        print_status "Found uninitialized submodules, initializing..."
        git submodule update --init --recursive
        print_success "Submodules initialized"
    else
        print_success "All submodules are already initialized"
    fi
}

# Function to list all submodules
list_submodules() {
    print_status "Current submodules:"
    git submodule status | while read -r line; do
        if [[ $line =~ ^[+-]?\ *([a-f0-9]+)\ +([^\ ]+)\ +(.+)$ ]]; then
            local status="${BASH_REMATCH[1]}"
            local path="${BASH_REMATCH[2]}"
            local name="${BASH_REMATCH[3]}"
            
            if [[ $status == +* ]]; then
                echo -e "  ${YELLOW}⚠${NC}  $path ($name) - not initialized"
            elif [[ $status == -* ]]; then
                echo -e "  ${RED}✗${NC}  $path ($name) - not initialized"
            else
                echo -e "  ${GREEN}✓${NC}  $path ($name)"
            fi
        fi
    done
}

# Main execution
main() {
    print_status "Starting vendored tools update..."
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    # Check if we're in a git repository
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        print_error "Not in a git repository"
        exit 1
    fi
    
    # Initialize submodules if needed
    init_submodules
    
    # List current submodules
    list_submodules
    
    # Find all submodules in tools directory
    local tools_submodules=()
    while IFS= read -r -d '' submodule; do
        if [[ "$submodule" == tools/* ]]; then
            tools_submodules+=("$submodule")
        fi
    done < <(git submodule foreach -q 'echo "$name"' 2>/dev/null || true)
    
    if [ ${#tools_submodules[@]} -eq 0 ]; then
        print_status "No submodules found in tools directory"
        print_status "To add a submodule, use: git submodule add <repo-url> tools/<name>"
        exit 0
    fi
    
    # Update each submodule
    local failed_updates=0
    for submodule in "${tools_submodules[@]}"; do
        if ! update_submodule "$submodule"; then
            ((failed_updates++))
        fi
    done
    
    # Summary
    echo
    if [ $failed_updates -eq 0 ]; then
        print_success "All vendored tools updated successfully!"
    else
        print_warning "$failed_updates tool(s) failed to update"
        exit 1
    fi
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [OPTIONS]"
        echo
        echo "Options:"
        echo "  --help, -h    Show this help message"
        echo "  --list, -l    List current submodules only"
        echo
        echo "This script updates all git submodules in the tools directory."
        exit 0
        ;;
    --list|-l)
        cd "$PROJECT_ROOT"
        list_submodules
        exit 0
        ;;
    "")
        main
        ;;
    *)
        print_error "Unknown option: $1"
        echo "Use --help for usage information"
        exit 1
        ;;
esac 