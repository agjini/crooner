# Crooner

![crooner](crooner.png)

A lightweight Docker-native cron scheduler written in Rust. Execute commands in other containers on schedule.

## Features

- Execute commands in Docker containers via Docker API
- Standard cron expressions with second precision
- Optional run-on-startup for immediate execution
- Capture and save command output to files
- Structured logging with tracing
- Secure non-root execution

## Quick Start

### Configuration

Create a `config.toml` file:

```toml
[[jobs]]
name = "PostgreSQL Backup"
at = "0 0 2 * * *"  # Every day at 2:00 AM
container = "postgres-container"
command = ["pg_dump", "-U", "postgres", "mydb"]
output_file = "/backups/postgres_backup.sql"
run_on_startup = true
```

### Docker Compose

```yaml
version: '3.8'

services:
  crooner:
    build: .
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - ./config.toml:/app/config.toml:ro
      - ./backups:/backups
    restart: unless-stopped
```

### Run

```bash
docker-compose up -d
```

## Configuration

### Job Fields

- `name`: Job identifier (string)
- `at`: Cron expression (format: `second minute hour day month weekday`)
- `container`: Target container name (string)
- `command`: Command to execute (array of strings)
- `output_file`: Optional output file path (string)
- `run_on_startup`: Execute immediately on startup (boolean, default: false)

### Cron Expression Format

```
┌───────────── second (0-59)
│ ┌─────────── minute (0-59)
│ │ ┌───────── hour (0-23)
│ │ │ ┌─────── day (1-31)
│ │ │ │ ┌───── month (1-12)
│ │ │ │ │ ┌─── weekday (0-6, 0=Sunday)
│ │ │ │ │ │
* * * * * *
```

Examples:

- `0 0 2 * * *` - Every day at 2:00 AM
- `0 */15 * * * *` - Every 15 minutes
- `0 0 0 1 * *` - First day of every month at midnight

## Logging

Control log level with `RUST_LOG` environment variable:

```yaml
environment:
  - RUST_LOG=info  # info (default), debug, trace
```

## Use Cases

- Database backups (PostgreSQL, MySQL, MongoDB)
- Cleanup and maintenance tasks
- Data exports and archiving
- Health checks and monitoring

## Building

```bash
cargo build --release
```

## License

MIT
