#!/usr/bin/env bash
#
# rewrite-history.sh - Rewrite Git commit history to use personal account
#
# This script rewrites ALL commits in the repository to use:
#   Name: Andrew Kroh
#   Email: id-github@andrewkroh.com
#
# WARNING: This rewrites Git history. All commit SHAs will change.
# After running this, collaborators will need to re-clone the repository.
#

set -euo pipefail

# Configuration
NEW_NAME="Andrew Kroh"
NEW_EMAIL="id-github@andrewkroh.com"
BACKUP_BRANCH="backup-before-rewrite"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $*"
}

error() {
    echo -e "${RED}[ERROR]${NC} $*"
    exit 1
}

# Function to show current authors/committers
show_current_authors() {
    info "Current unique authors in repository:"
    git log --all --format='%an <%ae>' | sort -u | sed 's/^/  /'
    echo
    info "Current unique committers in repository:"
    git log --all --format='%cn <%ce>' | sort -u | sed 's/^/  /'
    echo
}

# Function to preview what will change
preview_changes() {
    info "Preview of changes (first 10 commits):"
    echo
    git log --pretty=format:"  %h | OLD: %an <%ae> | %s" | head -10
    echo
    echo
    info "After rewrite, ALL commits will have:"
    echo "  Author:    $NEW_NAME <$NEW_EMAIL>"
    echo "  Committer: $NEW_NAME <$NEW_EMAIL>"
    echo
    info "Total commits to rewrite: $(git rev-list --all --count)"
    echo
}

# Function to create backup
create_backup() {
    local current_branch
    current_branch=$(git branch --show-current)

    info "Creating backup branch: $BACKUP_BRANCH"

    # Delete backup branch if it exists
    if git show-ref --verify --quiet "refs/heads/$BACKUP_BRANCH"; then
        warning "Backup branch $BACKUP_BRANCH already exists. Deleting it."
        git branch -D "$BACKUP_BRANCH"
    fi

    # Create backup from current HEAD
    git branch "$BACKUP_BRANCH" HEAD
    success "Backup branch created: $BACKUP_BRANCH"
    echo
}

# Function to perform the rewrite using filter-branch
rewrite_with_filter_branch() {
    info "Starting Git history rewrite using filter-branch..."
    info "This may take a few moments..."
    echo

    # Use filter-branch to rewrite all commits
    FILTER_BRANCH_SQUELCH_WARNING=1 git filter-branch --env-filter "
        export GIT_AUTHOR_NAME='$NEW_NAME'
        export GIT_AUTHOR_EMAIL='$NEW_EMAIL'
        export GIT_COMMITTER_NAME='$NEW_NAME'
        export GIT_COMMITTER_EMAIL='$NEW_EMAIL'
    " --tag-name-filter cat -- --all

    success "Git history rewrite completed!"
    echo
}

# Function to perform the rewrite using git-filter-repo (if available)
rewrite_with_filter_repo() {
    info "Starting Git history rewrite using git-filter-repo..."
    info "This may take a few moments..."
    echo

    # Create a mailmap for git-filter-repo
    local mailmap_file=".mailmap-temp"
    cat > "$mailmap_file" <<EOF
$NEW_NAME <$NEW_EMAIL> <andrew.kroh@elastic.co>
$NEW_NAME <$NEW_EMAIL> <claude@anthropic.com>
$NEW_NAME <$NEW_EMAIL> Claude <claude@anthropic.com>
$NEW_NAME <$NEW_EMAIL> Andrew Kroh <andrew.kroh@elastic.co>
EOF

    git filter-repo --mailmap "$mailmap_file" --force

    rm -f "$mailmap_file"

    success "Git history rewrite completed!"
    echo
}

# Function to verify the rewrite
verify_rewrite() {
    info "Verifying rewrite..."
    echo

    local unique_authors
    local unique_committers

    unique_authors=$(git log --all --format='%an <%ae>' | sort -u | wc -l)
    unique_committers=$(git log --all --format='%cn <%ce>' | sort -u | wc -l)

    info "Unique authors after rewrite:"
    git log --all --format='%an <%ae>' | sort -u | sed 's/^/  /'
    echo

    info "Unique committers after rewrite:"
    git log --all --format='%cn <%ce>' | sort -u | sed 's/^/  /'
    echo

    if [[ $unique_authors -eq 1 && $unique_committers -eq 1 ]]; then
        success "Verification passed! All commits now use: $NEW_NAME <$NEW_EMAIL>"
    else
        warning "Verification found multiple authors/committers. Please review manually."
    fi
    echo
}

# Function to show next steps
show_next_steps() {
    warning "IMPORTANT: Next Steps"
    echo
    echo "1. Review the changes:"
    echo "   git log --pretty=format:'%h %an <%ae> %s' | head -20"
    echo
    echo "2. Compare with backup:"
    echo "   git log $BACKUP_BRANCH --pretty=format:'%h %an <%ae> %s' | head -20"
    echo
    echo "3. If everything looks good, force push to remote:"
    echo "   git push --force --all origin"
    echo "   git push --force --tags origin"
    echo
    echo "4. Notify collaborators to re-clone the repository:"
    echo "   - They should NOT try to pull/merge"
    echo "   - They should backup their work and re-clone"
    echo "   - Any open PRs will need to be recreated"
    echo
    echo "5. If something went wrong, restore from backup:"
    echo "   git reset --hard $BACKUP_BRANCH"
    echo
    warning "Remember: This changes all commit SHAs. Anyone with a clone will need to re-clone."
    echo
}

# Main function
main() {
    local mode="${1:-preview}"

    # Check if we're in a git repository
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        error "Not in a Git repository!"
    fi

    # Check for uncommitted changes
    if ! git diff-index --quiet HEAD -- 2>/dev/null; then
        error "You have uncommitted changes. Please commit or stash them first."
    fi

    echo "======================================================================"
    echo "Git History Rewrite Script"
    echo "======================================================================"
    echo
    echo "This script will rewrite ALL commits to use:"
    echo "  Name:  $NEW_NAME"
    echo "  Email: $NEW_EMAIL"
    echo

    case "$mode" in
        preview)
            info "Running in PREVIEW mode (no changes will be made)"
            echo
            show_current_authors
            preview_changes
            echo "======================================================================"
            info "To perform the actual rewrite, run:"
            echo "  $0 rewrite"
            echo "======================================================================"
            ;;

        rewrite)
            warning "Running in REWRITE mode - this WILL modify Git history!"
            echo
            show_current_authors
            preview_changes

            echo "======================================================================"
            warning "This will PERMANENTLY rewrite Git history!"
            warning "All commit SHAs will change!"
            warning "Collaborators will need to re-clone the repository!"
            echo "======================================================================"
            echo

            read -p "Are you sure you want to continue? (type 'yes' to confirm): " confirm
            if [[ "$confirm" != "yes" ]]; then
                info "Aborted by user."
                exit 0
            fi
            echo

            create_backup

            # Try git-filter-repo first (faster), fall back to filter-branch
            if command -v git-filter-repo &> /dev/null; then
                info "git-filter-repo detected, using it for faster rewrite"
                rewrite_with_filter_repo
            else
                info "Using git filter-branch (consider installing git-filter-repo for faster rewrites)"
                rewrite_with_filter_branch
            fi

            verify_rewrite
            show_next_steps

            success "History rewrite complete!"
            ;;

        *)
            error "Unknown mode: $mode. Use 'preview' or 'rewrite'"
            ;;
    esac
}

# Run main function
main "$@"
