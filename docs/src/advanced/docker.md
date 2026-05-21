# Docker Deployment

AutoGPT ships two Docker images: `kevinrsdev/autogpt` for the agent binary and `kevinrsdev/orchgpt` for the orchestrator. A `docker-compose.yml` in the repository root wires them together.

## Pre-built Images

| Image                | Tag     | Size   | Purpose                |
| -------------------- | ------- | ------ | ---------------------- |
| `kevinrsdev/autogpt` | `0.4.5` | ~12 MB | `autogpt` agent CLI    |
| `kevinrsdev/orchgpt` | `0.4.5` | ~12 MB | `orchgpt` orchestrator |

Both images are Alpine-based and stripped of debug symbols for minimal footprint.

## Running a Single Agent Container

```sh
docker run -it \
  -e GEMINI_API_KEY=<your_key> \
  -e AUTOGPT_WORKSPACE=/workspace \
  -v $(pwd)/workspace:/workspace \
  --rm --name autogpt \
  kevinrsdev/autogpt
```

The `-v` flag mounts a local directory so generated files persist after the container exits.

## Running the Orchestrator Container

```sh
docker run -it \
  -e GEMINI_API_KEY=<your_key> \
  -p 8443:8443 \
  --rm --name orchgpt \
  kevinrsdev/orchgpt
```

## Docker Compose (Recommended)

The `docker-compose.yml` in the repository root starts both services with shared networking:

```yaml
services:
  autogpt:
    build:
      context: .
      dockerfile: Dockerfile.autogpt
    environment:
      - GEMINI_API_KEY=${GEMINI_API_KEY}
      - ORCHESTRATOR_ADDRESS=orchgpt:8443

  orchgpt:
    build:
      context: .
      dockerfile: Dockerfile.orchgpt
    environment:
      - GEMINI_API_KEY=${GEMINI_API_KEY}
    ports:
      - "8443:8443"
```

Start both:

```sh
docker compose up --build
```

Docker Compose sets up a bridge network so `autogpt` can reach `orchgpt` by container name. The `ORCHESTRATOR_ADDRESS=orchgpt:8443` env var tells the agent where to connect.

## Building Custom Images

To build with different feature flags, edit the Dockerfile and rebuild:

```sh
# Step 1: build autogpt image
docker build -f Dockerfile.autogpt -t my-autogpt .

# Step 2: build orchgpt image
docker build -f Dockerfile.orchgpt -t my-orchgpt .

# Step 3: run
docker run -i \
  -e GEMINI_API_KEY=<key> \
  -t my-autogpt
```

## Accessing the Container Workspace

After starting a container:

```sh
# Find running container ID
docker ps

# Attach shell
docker exec -it <container_id> /bin/sh

# Explore workspace
ls workspace/
# architect/  backend/  designer/  frontend/
```

Stop all running containers:

```sh
docker stop $(docker ps -q)
```
