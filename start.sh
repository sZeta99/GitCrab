#!/bin/bash

# Function to log messages
log_message() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') [STARTUP] \\$1"
}

log_message "Starting GitCrab SSH Git hosting service..."

# Ensure proper permissions
chown -R git:git /home/git
chmod 700 /home/git/.ssh
chmod 600 /home/git/.ssh/authorized_keys
chmod 755 /home/git/repositories
chmod 755 /home/git/git-shell-commands
chmod +x /home/git/git-shell-commands/git-serve
ln -s /home/git/repositories /repositories

# Start SSH server
log_message "Starting SSH server..."
/usr/sbin/sshd -D &
SSH_PID=$!

# Wait for SSH to start
sleep 2

# Check if SSH is running
if kill -0 $SSH_PID 2>/dev/null; then
    log_message "SSH server started successfully (PID: $SSH_PID)"
else
    log_message "ERROR: SSH server failed to start"
    exit 1
fi

# Start the main application
log_message "Starting GitCrab application..."
exec /usr/app/gitcrab-cli start