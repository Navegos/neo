# Docker CLI & Compose YAML Full Reference

A comprehensive reference guide for the Docker Command Line Interface (CLI) and Docker Compose YAML (`compose.yaml` / `docker-compose.yml`) specification, organized by functional area.

---

## 1. System & Information

Commands for inspecting the Docker daemon, version details, and system-wide resource usage.

| Command | Description |
| :--- | :--- |
| `docker version` | Show the Docker client and server version information. |
| `docker info` | Display system-wide information (containers, images, storage driver, etc.). |
| `docker system df` | Show Docker disk usage (images, containers, local volumes, build cache). |
| `docker system events` | Get real-time events from the server (`--since`, `--until`, `--filter`). |
| `docker system info` | Alias for `docker info`. |
| `docker system prune` | Remove unused data (stopped containers, unused networks, dangling images/cache). Add `-a` to remove all unused images, and `--volumes` for volumes. |
| `docker login [SERVER]` | Log in to a Docker registry (defaults to Docker Hub). |
| `docker logout [SERVER]` | Log out from a Docker registry. |
| `docker search <TERM>` | Search the Docker Hub for images. |

---

## 2. Container Lifecycle Management

Commands for creating, running, stopping, and deleting containers.

### Creating & Running
* **`docker run [OPTIONS] IMAGE [COMMAND] [ARG...]`**
  Create and start a new container from an image.
  * `-d, --detach`: Run container in background and print container ID.
  * `-it`: Keep STDIN open (`-i`) and allocate a pseudo-TTY (`-t`) for interactive shells.
  * `--name <NAME>`: Assign a custom name to the container.
  * `--rm`: Automatically remove the container when it exits.
  * `-p, --publish <HOST_PORT>:<CONTAINER_PORT>`: Publish a container's port(s) to the host.
  * `-P, --publish-all`: Publish all exposed ports to random ephemeral ports.
  * `-v, --volume <HOST_PATH>:<CONTAINER_PATH>`: Bind mount a volume or named volume.
  * `--mount type=bind|volume|tmpfs,source=...,target=...`: Attach a filesystem mount (more explicit syntax than `-v`).
  * `-e, --env <KEY>=<VALUE>`: Set environment variables.
  * `--env-file <FILE>`: Read in a file of environment variables.
  * `--network <NETWORK>`: Connect a container to a network.
  * `--restart <POLICY>`: Restart policy (`no`, `on-failure[:max-retries]`, `always`, `unless-stopped`).
  * `-u, --user <USER>`: Username or UID (format: `<name|uid>[:<group|gid>]`).
  * `-w, --workdir <DIR>`: Working directory inside the container.
  * `--memory <BYTES>` / `--cpus <NUMBER>`: Set resource limits.

* **`docker create [OPTIONS] IMAGE [COMMAND]`**: Create a new container without starting it.

### Starting, Stopping & Removing
| Command | Description |
| :--- | :--- |
| `docker start [OPTIONS] CONTAINER...` | Start one or more stopped containers (`-a` to attach, `-i` interactive). |
| `docker stop [OPTIONS] CONTAINER...` | Stop one or more running containers gracefully (`SIGTERM`, then `SIGKILL` after `-t` seconds). |
| `docker restart [OPTIONS] CONTAINER...` | Restart one or more containers. |
| `docker kill [OPTIONS] CONTAINER...` | Kill one or more running containers immediately (`-s` to specify signal). |
| `docker pause CONTAINER...` | Pause all processes within one or more containers. |
| `docker unpause CONTAINER...` | Unpause all processes within one or more containers. |
| `docker rm [OPTIONS] CONTAINER...` | Remove one or more containers (`-f` force remove running, `-v` remove anonymous volumes). |
| `docker container prune` | Remove all stopped containers. |
| `docker rename <OLD_NAME> <NEW_NAME>` | Rename a container. |
| `docker update [OPTIONS] CONTAINER...` | Update configuration of one or more containers (e.g., `--restart`, `--memory`). |

---

## 3. Container Inspection & Interaction

Commands for viewing container state, logs, processes, and executing commands inside running containers.

| Command | Description |
| :--- | :--- |
| `docker ps [OPTIONS]` | List running containers (`-a` all containers, `-q` only IDs, `-s` display file sizes, `-f` filter). |
| `docker exec [OPTIONS] CONTAINER COMMAND [ARG...]` | Run a command in a running container (e.g., `docker exec -it my_app /bin/bash`). |
| `docker attach [OPTIONS] CONTAINER` | Attach local standard input, output, and error streams to a running container. |
| `docker logs [OPTIONS] CONTAINER` | Fetch the logs of a container (`-f` follow, `--tail <N>` last N lines, `-t` timestamps, `--since`). |
| `docker inspect [OPTIONS] NAME\|ID...` | Return low-level information on Docker objects in JSON format (`-f` Go template format). |
| `docker top CONTAINER [ps OPTIONS]` | Display the running processes of a container. |
| `docker stats [OPTIONS] [CONTAINER...]` | Display a live stream of container(s) resource usage statistics (`--no-stream` for single snapshot). |
| `docker port CONTAINER [PRIVATE_PORT]` | List port mappings or a specific mapping for the container. |
| `docker diff CONTAINER` | Inspect changes to files or directories on a container's filesystem (`A` added, `D` deleted, `C` changed). |
| `docker cp <SRC_PATH> <CONTAINER>:<DEST_PATH>` | Copy files/folders between a container and the local filesystem (works in both directions). |
| `docker wait CONTAINER...` | Block until one or more containers stop, then print their exit codes. |
| `docker commit [OPTIONS] CONTAINER [REPO[:TAG]]` | Create a new image from a container's changes. |
| `docker export [OPTIONS] CONTAINER` | Export a container's filesystem as a tar archive (`-o` output file). |

---

## 4. Image Management

Commands for building, pulling, pushing, tagging, and managing Docker images.

| Command | Description |
| :--- | :--- |
| `docker images [OPTIONS] [REPO[:TAG]]` | List images (`-a` show intermediate images, `-q` only IDs, `--digests`). |
| `docker pull [OPTIONS] NAME[:TAG\|@DIGEST]` | Pull an image or a repository from a registry (`-a` all tags, `--platform`). |
| `docker push [OPTIONS] NAME[:TAG]` | Push an image or a repository to a registry. |
| `docker build [OPTIONS] PATH \| URL \| -` | Build an image from a Dockerfile (`-t name:tag`, `-f Dockerfile`, `--no-cache`, `--build-arg KEY=VAL`, `--target <STAGE>`). |
| `docker tag <SOURCE_IMAGE[:TAG]> <TARGET_IMAGE[:TAG]>` | Create a tag `TARGET_IMAGE` that refers to `SOURCE_IMAGE`. |
| `docker rmi [OPTIONS] IMAGE...` | Remove one or more images (`-f` force removal). |
| `docker image prune [OPTIONS]` | Remove unused images (`-a` remove all unused images, not just dangling ones). |
| `docker history [OPTIONS] IMAGE` | Show the history of an image (layers, commands, sizes). |
| `docker save [OPTIONS] IMAGE...` | Save one or more images to a tar archive (`-o image.tar`). |
| `docker load [OPTIONS]` | Load an image from a tar archive or STDIN (`-i image.tar`). |
| `docker import [OPTIONS] file\|URL\|- [REPO[:TAG]]` | Import the contents from a tarball to create a filesystem image. |

---

## 5. Buildx (Advanced Builds)

Commands for multi-platform builds and advanced BuildKit features.

| Command | Description |
| :--- | :--- |
| `docker buildx create [OPTIONS]` | Create a new builder instance. |
| `docker buildx use <BUILDER>` | Set the current builder instance. |
| `docker buildx ls` | List builder instances. |
| `docker buildx build [OPTIONS] PATH` | Start a build (supports `--platform linux/amd64,linux/arm64`, `--push`, `--load`). |
| `docker buildx bake [OPTIONS] [TARGET...]` | Build from a `docker-compose.yml` or HCL/JSON definition file. |
| `docker buildx prune` | Remove build cache. |

---

## 6. Networking

Commands for managing Docker networks and container connectivity.

| Command | Description |
| :--- | :--- |
| `docker network ls` | List networks. |
| `docker network create [OPTIONS] NETWORK` | Create a network (`-d bridge\|overlay\|host\|macvlan\|none`, `--subnet`, `--gateway`). |
| `docker network inspect NETWORK...` | Display detailed information on one or more networks. |
| `docker network connect [OPTIONS] NETWORK CONTAINER` | Connect a container to a network (`--alias`, `--ip`). |
| `docker network disconnect [OPTIONS] NETWORK CONTAINER` | Disconnect a container from a network (`-f` force). |
| `docker network rm NETWORK...` | Remove one or more networks. |
| `docker network prune` | Remove all unused networks. |

---

## 7. Volumes & Storage

Commands for managing persistent data storage outside container lifecycles.

| Command | Description |
| :--- | :--- |
| `docker volume ls` | List volumes (`-f dangling=true`). |
| `docker volume create [OPTIONS] [VOLUME]` | Create a volume (`-d` driver, `-o` driver-specific options). |
| `docker volume inspect VOLUME...` | Display detailed information on one or more volumes (including mountpoint on host). |
| `docker volume rm VOLUME...` | Remove one or more volumes (`-f` force). |
| `docker volume prune` | Remove all unused local volumes (`-a` remove all unused volumes, including named ones). |

---

## 8. Docker Compose (V2 CLI Plugin)

Commands for defining and running multi-container applications using `compose.yaml` / `docker-compose.yml`.

| Command | Description |
| :--- | :--- |
| `docker compose up [OPTIONS] [SERVICE...]` | Create and start containers (`-d` detached, `--build` rebuild images, `--force-recreate`, `--scale SERVICE=N`). |
| `docker compose down [OPTIONS]` | Stop and remove containers, networks (`-v` remove named volumes, `--rmi all\|local` remove images). |
| `docker compose ps [OPTIONS]` | List containers for the Compose project (`-a` include stopped). |
| `docker compose logs [OPTIONS] [SERVICE...]` | View output from containers (`-f` follow, `--tail N`). |
| `docker compose exec [OPTIONS] SERVICE COMMAND` | Execute a command in a running service container. |
| `docker compose run [OPTIONS] SERVICE [COMMAND]` | Run a one-off command on a service (`--rm` remove container after run). |
| `docker compose build [OPTIONS] [SERVICE...]` | Build or rebuild services (`--no-cache`, `--pull`). |
| `docker compose pull [OPTIONS] [SERVICE...]` | Pull service images. |
| `docker compose push [OPTIONS] [SERVICE...]` | Push service images. |
| `docker compose start [SERVICE...]` | Start existing service containers. |
| `docker compose stop [SERVICE...]` | Stop running service containers without removing them. |
| `docker compose restart [SERVICE...]` | Restart service containers. |
| `docker compose config [OPTIONS]` | Validate and view the resolved Compose file (`--services`, `--volumes`). |
| `docker compose top [SERVICE...]` | Display the running processes. |

---

## 9. Docker Compose YAML (`compose.yaml`) Reference

The Compose Specification defines the YAML schema (`compose.yaml` preferred, or `docker-compose.yml`) used to configure multi-container applications. *(Note: The top-level `version:` property is obsolete in modern Compose V2).*

### Top-Level Elements
| Key | Description |
| :--- | :--- |
| `name` | Defines the project name explicitly (overrides directory name or `-p`). |
| `services` | Map of application containers/services to run. |
| `networks` | Defines custom networks to be created or referenced as `external`. |
| `volumes` | Defines named volumes to persist data across container restarts. |
| `configs` | Stores non-sensitive configuration files mounted into containers. |
| `secrets` | Stores sensitive data (passwords, API keys) mounted securely at `/run/secrets/<name>`. |
| `include` | Imports other Compose files into the current application model. |

### Service Configuration Keys (`services.<name>`)

#### Image & Build
* **`image`**: Image name and tag to pull or use when building (e.g., `nginx:alpine`).
* **`build`**: Build configuration (string path or object):
  * `context`: Path to the build directory or Git URL.
  * `dockerfile`: Alternate Dockerfile name.
  * `args`: Build-time variables (`KEY: value`).
  * `target`: Multi-stage build target stage name.
  * `platforms`: Target platforms (e.g., `["linux/amd64", "linux/arm64"]`).
  * `cache_from` / `cache_to`: External cache sources/destinations.

#### Execution & Lifecycle
* **`container_name`**: Custom container name (prevents scaling the service beyond 1 replica).
* **`command`**: Overrides the default `CMD` specified by the image (string or list).
* **`entrypoint`**: Overrides the default `ENTRYPOINT` specified by the image.
* **`working_dir`**: Sets the working directory inside the container.
* **`user`**: Sets the UID/username (`"1000:1000"`).
* **`restart`**: Restart policy (`"no"`, `"always"`, `"on-failure"`, `"unless-stopped"`).
* **`init`**: Run an init process (`tini`) inside the container to forward signals and reap zombie processes (`true`/`false`).
* **`profiles`**: List of profile names; service only starts when matching `--profile` is active.

#### Dependencies & Healthchecks
* **`depends_on`**: Controls startup and shutdown order:
  * Short syntax: List of service names (`- db`).
  * Long syntax: Object specifying `condition` (`service_started`, `service_healthy`, `service_completed_successfully`) and `restart: true`.
* **`healthcheck`**: Configures container health verification:
  * `test`: Command to run (e.g., `["CMD", "curl", "-f", "http://localhost/health"]`).
  * `interval`: Time between checks (e.g., `30s`).
  * `timeout`: Time to wait for a single check (e.g., `10s`).
  * `retries`: Consecutive failures needed to mark `unhealthy` (e.g., `3`).
  * `start_period`: Initialization grace period before failures count (e.g., `40s`).

#### Networking & Ports
* **`ports`**: Exposes ports (`"HOST:CONTAINER"` or long syntax with `target`, `published`, `protocol`, `mode`).
* **`expose`**: Exposes ports to linked/networked services only, without publishing to the host.
* **`networks`**: List of networks to join, or object specifying `aliases` and static `ipv4_address`.
* **`network_mode`**: Sets network mode (`"bridge"`, `"host"`, `"none"`, `"service:<name>"`).
* **`dns`**: Custom DNS servers (string or list).
* **`extra_hosts`**: Adds hostname mappings to `/etc/hosts` (`"host.docker.internal:host-gateway"`).

#### Environment & Secrets
* **`environment`**: Sets environment variables inside the container (map `KEY: val` or list `KEY=val`).
* **`env_file`**: Loads environment variables from one or more `.env` files.
* **`secrets`**: Grants access to top-level `secrets` (mounted at `/run/secrets/<secret_name>`).
* **`configs`**: Grants access to top-level `configs`.

#### Storage & Volumes
* **`volumes`**: Mounts host paths or named volumes:
  * Short syntax: `HOST_PATH:CONTAINER_PATH[:ro|rw]` or `VOLUME_NAME:CONTAINER_PATH`.
  * Long syntax: Object with `type` (`volume`, `bind`, `tmpfs`), `source`, `target`, `read_only`.
* **`tmpfs`**: Mounts temporary in-memory filesystems (`- /run`).

#### Resource Limits & Deployment (`deploy`)
* **`deploy`**: Deployment and runtime constraints (supported in both Swarm and standalone Compose):
  * `replicas`: Number of container instances to run.
  * `resources.limits`: Maximum `cpus` (e.g., `'0.50'`) and `memory` (e.g., `512M`).
  * `resources.reservations`: Guaranteed `cpus`, `memory`, or `devices` (e.g., NVIDIA GPUs).
  * `restart_policy`: Fine-grained restart control (`condition`, `delay`, `max_attempts`, `window`).

#### Development & Logging
* **`develop.watch`**: Automatic file sync/rebuild on source changes (`docker compose watch`) with `action` (`sync`, `rebuild`, `sync+restart`), `path`, and `target`.
* **`logging`**: Logging driver configuration (`driver: json-file` and `options` like `max-size: "10m"`, `max-file: "3"`).
* **`stdin_open`** / **`tty`**: Equivalent to `docker run -i` and `-t`.

### Complete `compose.yaml` Example

```yaml
name: my-web-stack

services:
  web:
    build:
      context: ./app
      dockerfile: Dockerfile
      target: production
      args:
        NODE_ENV: production
    image: my-org/web-app:latest
    restart: unless-stopped
    ports:
      - "8080:3000"
    environment:
      DATABASE_URL: postgres://app_user@db:5432/app_db
      REDIS_HOST: cache
    env_file:
      - .env
    secrets:
      - db_password
    volumes:
      - ./app/public:/usr/src/app/public:ro
      - app_uploads:/usr/src/app/uploads
    networks:
      - frontend
      - backend
    depends_on:
      db:
        condition: service_healthy
      cache:
        condition: service_started
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 15s
    deploy:
      resources:
        limits:
          cpus: "1.0"
          memory: 512M
    logging:
      driver: json-file
      options:
        max-size: "10m"
        max-file: "3"

  db:
    image: postgres:16-alpine
    restart: unless-stopped
    environment:
      POSTGRES_USER: app_user
      POSTGRES_DB: app_db
      POSTGRES_PASSWORD_FILE: /run/secrets/db_password
    secrets:
      - db_password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    networks:
      - backend
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U app_user -d app_db"]
      interval: 10s
      timeout: 5s
      retries: 5

  cache:
    image: redis:7-alpine
    restart: unless-stopped
    command: ["redis-server", "--appendonly", "yes"]
    volumes:
      - redis_data:/data
    networks:
      - backend

networks:
  frontend:
    driver: bridge
  backend:
    driver: bridge
    internal: true

volumes:
  postgres_data:
  redis_data:
  app_uploads:

secrets:
  db_password:
    file: ./secrets/db_password.txt
```

---

## 10. Contexts

Commands for managing connections to multiple Docker daemons (local, remote SSH, cloud).

| Command | Description |
| :--- | :--- |
| `docker context ls` | List available contexts. |
| `docker context create <NAME> --docker "host=ssh://user@host"` | Create a new context. |
| `docker context use <NAME>` | Set the current default context. |
| `docker context inspect [CONTEXT...]` | Display detailed information on one or more contexts. |
| `docker context rm CONTEXT...` | Remove one or more contexts. |

---

## 10. Useful One-Liners & Recipes

Commonly used command combinations for maintenance and debugging:

* **Stop all running containers:**
  ```bash
  docker stop $(docker ps -q)
  ```
* **Remove all stopped containers:**
  ```bash
  docker rm $(docker ps -aq)
  ```
* **Remove all dangling (`<none>`) images:**
  ```bash
  docker rmi $(docker images -f "dangling=true" -q)
  ```
* **Get the IP address of a container:**
  ```bash
  docker inspect -f '{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}' <CONTAINER_NAME>
  ```
* **Follow logs for the last 100 lines with timestamps:**
  ```bash
  docker logs -f -t --tail 100 <CONTAINER_NAME>
  ```
* **Open an interactive root shell in a running container:**
  ```bash
  docker exec -it -u 0 <CONTAINER_NAME> /bin/sh
  ```
* **Full nuclear cleanup (containers, networks, all unused images, and volumes):**
  ```bash
  docker system prune -a --volumes
  ```