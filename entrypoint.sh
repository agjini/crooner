#!/bin/sh
set -e

# Get the GID of the mounted Docker socket
DOCKER_SOCKET=/var/run/docker.sock

if [ -S "$DOCKER_SOCKET" ]; then
    DOCKER_GID=$(stat -c '%g' "$DOCKER_SOCKET")
    echo "Docker socket found with GID: $DOCKER_GID"

    # Check if docker group exists, if not create it
    if ! getent group docker > /dev/null 2>&1; then
        echo "Creating docker group with GID $DOCKER_GID"
        groupadd -g "$DOCKER_GID" docker
    else
        # Update existing docker group GID if different
        CURRENT_GID=$(getent group docker | cut -d: -f3)
        if [ "$CURRENT_GID" != "$DOCKER_GID" ]; then
            echo "Updating docker group GID from $CURRENT_GID to $DOCKER_GID"
            groupmod -g "$DOCKER_GID" docker
        fi
    fi

    # Add crooner user to docker group if not already a member
    if ! groups crooner | grep -q docker; then
        echo "Adding crooner user to docker group"
        usermod -aG docker crooner
    fi
else
    echo "Warning: Docker socket not found at $DOCKER_SOCKET"
    exit 1
fi

# Ensure backups directory is writable by crooner user
if [ -d /backups ]; then
    chown -R crooner:crooner /backups
    echo "Set permissions for /backups directory"
fi

# Switch to crooner user and execute the command
echo "Starting crooner as user 'crooner'..."
exec gosu crooner "$@"
