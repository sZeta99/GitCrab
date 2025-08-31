# Multi-stage build for efficient image size
FROM rust:1.83.0-slim as builder

# Install dependencies
RUN apt-get update && \
    apt-get install -y \
        pkg-config \
        libssl-dev && \
    rm -rf /var/lib/apt/lists/*


WORKDIR /usr/src/
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

# Install Git, SSH server, and required dependencies
RUN apt-get update && \
    apt-get install -y \
        git \
        openssh-server \
        curl \
        ca-certificates \
        sudo && \
    rm -rf /var/lib/apt/lists/*

# Configure SSH server
RUN mkdir -p /var/run/sshd && \
    ssh-keygen -A
RUN echo "Port 22" >> /etc/ssh/sshd_config && \
    echo "PermitRootLogin no" >> /etc/ssh/sshd_config && \
    echo "PasswordAuthentication no" >> /etc/ssh/sshd_config && \
    echo "PubkeyAuthentication yes" >> /etc/ssh/sshd_config && \
    echo "AuthorizedKeysFile %h/.ssh/authorized_keys" >> /etc/ssh/sshd_config && \
    echo "AllowUsers git" >> /etc/ssh/sshd_config

# Create git user with git-shell
RUN useradd -m -d /home/git -s /usr/bin/git-shell git && \
    mkdir -p /home/git/.ssh /home/git/repositories /home/git/git-shell-commands && \
    chmod 700 /home/git/.ssh && \
    chmod 755 /home/git/git-shell-commands && \
    touch /home/git/.ssh/authorized_keys && \
    chmod 600 /home/git/.ssh/authorized_keys && \
    chown -R git:git /home/git

# Copy scripts
COPY git-serve.sh /home/git/git-shell-commands/git-serve
COPY start.sh /usr/local/bin/start.sh

# Set permissions for scripts
RUN chmod +x /home/git/git-shell-commands/git-serve && \
    chmod +x /usr/local/bin/start.sh && \
    chown -R git:git /home/git/git-shell-commands

# Create log directory
RUN mkdir -p /var/log && \
    touch /var/log/git-access.log && \
    chmod 644 /var/log/git-access.log

# Copy application files
WORKDIR /usr/app
COPY --from=builder /usr/src/assets assets
COPY --from=builder /usr/src/config config
COPY --from=builder /usr/src/target/release/gitcrab-cli gitcrab-cli

# Expose ports
EXPOSE 5150 22

# Run startup script
ENTRYPOINT [ "/usr/local/bin/start.sh" ]
