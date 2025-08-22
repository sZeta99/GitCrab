#!/bin/bash

USERNAME="\$1"
REPO_BASE="/home/git/repositories"
LOG_FILE="/var/log/git-access.log"

# Create log file if it doesn't exist
touch "$LOG_FILE"
chmod 644 "$LOG_FILE"

# Log function
log_message() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') [$USERNAME] \$1" >> "$LOG_FILE"
}

log_message "SSH connection established - Command: $SSH_ORIGINAL_COMMAND"

# Validate SSH command
if [ -z "$SSH_ORIGINAL_COMMAND" ]; then
    log_message "ERROR: No SSH command provided"
    echo "No command provided"
    exit 1
fi

# Parse repository name
parse_repo_name() {
    local cmd="\$1"
    echo "$cmd" | sed -E "s/^git-(upload|receive)-pack '?([^']+)'\$/\2/" | sed 's/^\///'
}

# Check if repository exists
check_repo_exists() {
    local repo_path="\$1"
    if [ -d "$repo_path" ] && [ -f "$repo_path/HEAD" ]; then
        return 0
    else
        return 1
    fi
}

# Add safe directory exception
add_safe_directory() {
    local repo_path="\$1"
    sudo -u git git config --global --add safe.directory "$repo_path"
}

# Handle Git commands
case "$SSH_ORIGINAL_COMMAND" in
    "git-upload-pack "*)
        REPO_NAME=$(parse_repo_name "$SSH_ORIGINAL_COMMAND")
        REPO_PATH="$REPO_BASE/$REPO_NAME"
        log_message "UPLOAD-PACK request for repository: $REPO_NAME"

        if check_repo_exists "$REPO_PATH"; then
            log_message "Repository found, executing git-upload-pack"
            add_safe_directory "$REPO_PATH"
            sudo -u git git-upload-pack "$REPO_PATH"
        else
            log_message "ERROR: Repository not found: $REPO_PATH"
            echo "Repository '$REPO_NAME' not found"
            exit 1
        fi
        ;;

    "git-receive-pack "*)
        REPO_NAME=$(parse_repo_name "$SSH_ORIGINAL_COMMAND")
        REPO_PATH="$REPO_BASE/$REPO_NAME"
        log_message "RECEIVE-PACK request for repository: $REPO_NAME"

        if check_repo_exists "$REPO_PATH"; then
            log_message "Repository found, executing git-receive-pack"
            add_safe_directory "$REPO_PATH"
            sudo -u git git-receive-pack "$REPO_PATH"
        else
            log_message "ERROR: Repository not found: $REPO_PATH"
            echo "Repository '$REPO_NAME' not found"
            exit 1
        fi
        ;;

    *)
        log_message "ERROR: Invalid Git command: $SSH_ORIGINAL_COMMAND"
        echo "Invalid Git command. Only git-upload-pack and git-receive-pack are supported."
        echo "Received: $SSH_ORIGINAL_COMMAND"
        exit 1
        ;;
esac
