# `sbx` --- Docker Sandboxes CLI Reference

**Description:** Manage AI coding agent sandboxes.  
**Usage:** `sbx COMMAND`

Docker Sandboxes creates isolated sandbox environments for AI agents, powered by Docker. Run agents locally via `sandboxd` or remotely in Docker Cloud Sandboxes using `--cloud`.

---

## Global Options

These flags apply to `sbx` and its subcommands:

| Option | Default | Description |
| :--- | :--- | :--- |
| `--cloud` | | Dispatch to Docker Cloud Sandboxes API instead of local `sandboxd` (supported by a growing set of verbs --- run `sbx --cloud --help` for the current list) |
| `-D, --debug` | | Enable debug logging |
| `--no-input` | | Never prompt; commands that need an answer fail with the command to run instead |

---

## Top-Level Commands Summary

| Command | Description |
| :--- | :--- |
| `sbx attach` | Attach to a cloud sandbox, starting it first if it is stopped |
| `sbx completion` | Generate the autocompletion script for the specified shell |
| `sbx cp` | Copy files or directories between a sandbox and the host |
| `sbx create` | Create a sandbox for an agent or sandbox kit |
| `sbx daemon` | Manage the `sandboxd` daemon |
| `sbx df` | Show runtime disk usage |
| `sbx diagnose` | Diagnose common issues with your `sbx` installation |
| `sbx env` | Manage sandbox environments *(experimental)* |
| `sbx exec` | Execute a command inside a sandbox |
| `sbx inspect` | Show details for a sandbox |
| `sbx kit` | Manage kit artifacts |
| `sbx login` | Sign in to Docker |
| `sbx logout` | Sign out of Docker |
| `sbx ls` | List sandboxes |
| `sbx mcp` | Manage MCP servers |
| `sbx move` | Move a sandbox between local and cloud |
| `sbx policy` | Manage persistent access policies for sandboxes |
| `sbx ports` | Manage sandbox port publishing |
| `sbx prune` | Remove all stopped sandboxes |
| `sbx reset` | Reset all sandboxes and clean up state |
| `sbx rm` | Remove one or more sandboxes |
| `sbx run` | Run an agent in a sandbox |
| `sbx schedule` | Manage scheduled cloud sandbox runs *(cloud-only)* |
| `sbx secret` | Manage stored secrets |
| `sbx settings` | Inspect and manage Docker Sandboxes settings |
| `sbx setup` | Detect host configuration and prepare Docker Sandboxes *(experimental)* |
| `sbx skills` | Manage skills available in sandboxes *(experimental)* |
| `sbx stop` | Stop one or more sandboxes without removing them |
| `sbx template` | Manage sandbox templates |
| `sbx ttl` | Inspect or extend a cloud sandbox's TTL *(cloud-only)* |
| `sbx tui` | Open the interactive TUI dashboard |
| `sbx version` | Show Docker Sandboxes version information |
| `sbx volume` | Manage persistent volumes *(cloud-only)* |

---

## Core Lifecycle & Execution Commands

### `sbx run`
Run an agent in a sandbox (creates and attaches in one step, or reuses a matching sandbox).

- **Usage:** `sbx run [flags] [AGENT|SANDBOX_KIT] [PATH...] [-- AGENT_ARGS...]`
- **Key Options:**
  - `--allow-network`: Network pattern to allow for this sandbox
  - `-d, --detach`: Run sandbox in the background without attaching
  - `--detach-keys`: Override the detach key sequence (cloud only, default `Ctrl-\`)
  - `--kit`: Add mixin kits (repeatable; accepts local paths, ZIPs, Git URLs, or OCI references)
  - `--kit-arg`: Pass `name=value` or `kit.name=value` arguments to kits
  - `--kit-args-file`: Read kit arguments from a `name=value` file
  - `--model`: Select local or hosted model (e.g., `gemma4`)
  - `--name`: Name for the sandbox
  - `--provider`: Model provider (e.g., `ollama`)
  - `--pull`: Image pull policy (`always`, `missing`, `never`)
  - `--skills`: Skill mount mode (`readonly`, `readwrite`, `off`)
  - `-t, --template`: Custom template image reference
  - `--volume`: Mount a persistent volume (cloud)
- **Examples:**
  ```bash
  sbx run claude
  sbx run ./my-agent --name my-project --kit ./team-config
  sbx --cloud run claude --name cloud-dev
  ```

### `sbx create`
Create a sandbox for an agent or sandbox kit without attaching to it.

- **Usage:** `sbx create AGENT|SANDBOX_KIT [PATH...] [flags]`
- **Supported Built-in Agents / Subcommands:**
  - `sbx create claude` --- Create a sandbox for Claude Code
  - `sbx create codex` --- Create a sandbox for OpenAI Codex
  - `sbx create cursor` --- Create a sandbox for Cursor
  - `sbx create devin` --- Create a sandbox for Devin
  - `sbx create docker-agent` --- Create a sandbox for Docker Agent
  - `sbx create gemini` --- Create a sandbox for Gemini CLI
  - `sbx create opencode` --- Create a sandbox for OpenCode
  - `sbx create shell` --- Create a plain Bash shell sandbox without an agent
- **Key Options:**
  - `--allow-network`: Network pattern to allow for the new sandbox
  - `--kit`: Add mixin kits
  - `--kit-arg` / `--kit-args-file`: Pass arguments to kits
  - `--name`: Assign a custom sandbox name
  - `--skills`: Skill mount mode (`readonly`, `readwrite`, `off`)
  - `-t, --template`: Template image reference
- **Examples:**
  ```bash
  sbx create --name demo shell
  sbx create claude ~/my-project
  ```

### `sbx attach`
Attach an interactive terminal session to a cloud sandbox, starting it first if it is stopped. *(Cloud-only: requires `--cloud`)*

- **Usage:** `sbx attach SANDBOX [flags]`
- **Options:**
  - `--detach-keys`: Override the detach gesture that leaves the session running (default: `Ctrl-\`, e.g., `"ctrl-x,ctrl-d"`)
- **Examples:**
  ```bash
  sbx --cloud attach sbx_abc123
  sbx --cloud attach claude/my-sandbox
  ```

### `sbx exec`
Execute a command inside a sandbox. Starts the sandbox first if it is stopped.

- **Usage:** `sbx exec [flags] SANDBOX COMMAND [ARG...]`
- **Options:**
  - `--detach-keys`: Override the key sequence for detaching an interactive cloud exec session (not supported for local exec)
  - `-e, --env`: Set environment variables
  - `--env-file`: Read in a file of environment variables
  - `-i, --interactive`: Keep STDIN open even if not attached
  - `-t, --tty`: Allocate a pseudo-TTY
  - `-u, --user`: Username or UID (`<name|uid>[:<group|gid>]`, local only)
  - `-w, --workdir`: Working directory inside the container
- **Exit Status:** Returns command exit code; `125` if `sbx` failed to run the command; `127` if command is missing.
- **Examples:**
  ```bash
  sbx exec -it my-sandbox bash
  sbx exec -u root my-sandbox apt-get update
  sbx --cloud exec -it sbx_abc123 bash
  ```

### `sbx ls`
List all sandboxes with their agent, status, published ports, and workspace.

- **Usage:** `sbx ls [flags]`
- **Options:**
  - `--format`: Output format: `table`, `json`, or `yaml`
  - `--json`: Output in JSON format
  - `-q, --quiet`: Only display sandbox names
- **Examples:**
  ```bash
  sbx ls
  sbx ls -q
  sbx ls --format json
  ```

### `sbx inspect`
Show a human-readable summary of a sandbox (agent, kits, state, auth mode, workspace/mounts, network policy, secrets, published ports, active sessions, or cloud sizing/TTL/URLs).

- **Usage:** `sbx inspect SANDBOX [flags]`
- **Options:**
  - `--format`: Output format: `table`, `json`, or `yaml`
  - `--json`: Output in JSON format
- **Examples:**
  ```bash
  sbx inspect my-sandbox
  sbx --cloud inspect my-sandbox --format json
  ```

### `sbx stop`
Stop one or more running sandboxes without removing them. Stopped sandboxes retain their state and can be restarted with `sbx run` (or `sbx --cloud attach`).

- **Usage:** `sbx stop SANDBOX... [flags]`
- **Options:**
  - `--format`: Output format: `table`, `json`, or `yaml`
  - `--json`: Output in JSON format
- **Examples:**
  ```bash
  sbx stop my-sandbox
  sbx stop one two
  ```

### `sbx rm`
Remove one or more sandboxes (stops containers, cleans up Git worktrees, deletes sandbox state and scoped secrets; keeps host mounts and persistent volumes).

- **Usage:** `sbx rm [SANDBOX...] [flags]`
- **Options:**
  - `--all`: Remove all sandboxes *(disabled in `--cloud` mode as a safety gate)*
  - `-f, --force`: Skip confirmation prompts and delete even if in use (e.g., active SSH session)
  - `--format`: Output format: `table`, `json`, or `yaml`
  - `--json`: Output in JSON format
- **Examples:**
  ```bash
  sbx rm my-sandbox
  sbx rm one two
  sbx rm --all --force
  ```

### `sbx prune`
Remove all stopped sandboxes and their associated resources *(local-only)*.

- **Usage:** `sbx prune [flags]`
- **Options:**
  - `--dry-run`: List the sandboxes that would be removed without removing them
  - `--filter`: Filter candidates (supported: `until=TIMESTAMP` --- RFC 3339, Unix timestamp, or Go duration like `168h`)
  - `-f, --force`: Skip confirmation prompts and remove even if in use
  - `--format`: Output format: `table`, `json`, or `yaml`
  - `--json`: Output in JSON format
- **Examples:**
  ```bash
  sbx prune --dry-run
  sbx prune --force
  sbx prune --filter until=2026-01-01T00:00:00Z
  ```

### `sbx move`
Move a sandbox between the local host and Docker Cloud Sandboxes.

- **Usage:** `sbx move SANDBOX [flags]`

### `sbx cp`
Copy files or directories between a sandbox (`SANDBOX:/absolute/path`) and the host.

- **Usage:** `sbx cp SRC DST [flags]`
- **Options:**
  - `-L, --follow-link`: Follow symbolic links in the source path *(cloud: does not follow links inside downloaded directories)*
- **Examples:**
  ```bash
  sbx cp ./config.json my-sandbox:/home/user/
  sbx cp my-sandbox:/home/user/output.log ./
  sbx --cloud cp sbx_abc:/workspace/out.log ./
  ```

### `sbx ports`
Manage sandbox port publishing (list, publish, or unpublish ports).

- **Usage:** `sbx ports SANDBOX [flags]`

### `sbx ttl`
Inspect or extend a cloud sandbox's TTL up to the 24-hour creation ceiling *(cloud-only)*.

- **Usage:** `sbx ttl [+DURATION] SANDBOX [flags]`
- **Options:**
  - `--json`: Output as JSON
- **Examples:**
  ```bash
  sbx --cloud ttl sbx_abc123
  sbx --cloud ttl +2h sbx_abc123
  ```

### `sbx tui`
Open the interactive terminal UI dashboard.

- **Usage:** `sbx tui [flags]`

---

## Authentication, Setup & Settings

### `sbx login`
Sign in to Docker.

- **Usage:** `sbx login [flags]`
- **Options:**
  - `--password-stdin`: Read password or access token from STDIN

### `sbx logout`
Sign out of Docker.

- **Usage:** `sbx logout [flags]`

### `sbx setup` *(experimental)*
Detect host configuration (API keys in env vars, SSH agent sockets) and prepare Docker Sandboxes.

- **Usage:** `sbx setup [COMMAND]`
- **Subcommands:**
  - `sbx setup ssh`: Set up SSH client config (`*.sbx`) for sandbox endpoints (supports `--auto-create` for desktop apps)
  - `sbx setup ssh remove`: Remove managed SSH configuration and desktop app connections

### `sbx settings`
Inspect and manage persistent Docker Sandboxes settings (e.g., `browser.enabled`, `clipboard.imagePaste`, `sandbox.disk.dockerVolume`, `model.server.enabled`).

- **Usage:** `sbx settings COMMAND`
- **Subcommands:**
  - `sbx settings get KEY`: Get the effective value of a setting
  - `sbx settings ls`: List all settings (`--no-trunc`, `--json`)
  - `sbx settings set KEY VALUE`: Set a persistent override
  - `sbx settings unset KEY`: Remove a stored override

---

## Policies & Approvals (`sbx policy`)

Manage persistent network and access policies for sandboxes.

- **Usage:** `sbx policy COMMAND`
- **Subcommands:**
  - `sbx policy allow`: Add an allow rule for sandboxes
    - `sbx policy allow network`: Allow network access to specified host/port patterns
  - `sbx policy approval`: Manage pending policy approval requests
    - `sbx policy approval inspect`: Inspect a pending approval
    - `sbx policy approval ls`: List pending approvals
    - `sbx policy approval respond`: Respond to a pending approval (`--option`)
  - `sbx policy check`: Check whether policy allows an access request
    - `sbx policy check http`: Check an HTTP request against policy
    - `sbx policy check network`: Check a network host/port against policy
  - `sbx policy deny`: Add a deny rule for sandboxes
    - `sbx policy deny network`: Deny network access to specified host/port patterns
  - `sbx policy init`: Initialize the global network policy
  - `sbx policy inspect POLICY|RULE`: Show full policy or rule details (`--format`)
  - `sbx policy log`: Show sandbox policy logs
  - `sbx policy ls`: List sandbox policies
  - `sbx policy refresh`: Force an immediate policy refresh
  - `sbx policy reset`: Reset policies to defaults
  - `sbx policy rm`: Remove a policy rule
    - `sbx policy rm network`: Remove a network policy rule

---

## Secrets Management (`sbx secret`)

Manage proxy-injected **service secrets** (e.g., `github`, `anthropic`, `openai`) and host-only **registry secrets** (e.g., `ghcr.io`).

- **Usage:** `sbx secret COMMAND`
- **Subcommands:**
  - `sbx secret import`: Import secrets detected in host environment variables
  - `sbx secret ls`: List stored secrets
  - `sbx secret rm`: Remove a secret
  - `sbx secret set`: Create or update a secret (supports `--registry`, `--password-stdin`, `--sandbox`, `--all-sandboxes`)
  - `sbx secret set-custom` *(experimental)*: Create or update a custom secret

---

## Kits & Environments (`sbx kit` & `sbx env`)

### `sbx kit`
Manage sandbox kit artifacts and mixins.

- **Usage:** `sbx kit COMMAND`
- **Subcommands:**
  - `sbx kit add` *(experimental)*: Add a mixin kit to an existing sandbox
  - `sbx kit builder` *(experimental)*: Manage the kit builder sandbox
    - `sbx kit builder history`: Show kit build history (`export`, `inspect`, `logs`, `ls`, `rm`, `trace`)
    - `sbx kit builder rm`: Remove the kit builder sandbox
    - `sbx kit builder status`: Show kit builder status
  - `sbx kit inspect`: Inspect kit metadata, arguments, capabilities, and files
  - `sbx kit pack`: Package a kit directory into a distributable archive
  - `sbx kit provenance`: Inspect or manage kit build provenance
  - `sbx kit pull`: Pull a kit artifact from an OCI registry
  - `sbx kit push`: Push a kit artifact to an OCI registry
  - `sbx kit sign`: Sign a kit artifact
  - `sbx kit validate`: Validate a kit directory or archive
  - `sbx kit verify`: Verify a signed kit artifact

### `sbx env` *(experimental)*
Manage declarative sandbox environments.

- **Usage:** `sbx env COMMAND`
- **Subcommands:**
  - `sbx env create`: Create the environment described in the target directory
  - `sbx env exec`: Execute a command inside a sandbox environment
  - `sbx env plan`: Preview what an environment would change without applying it
  - `sbx env rm`: Remove an environment and optionally its credential bindings
  - `sbx env run`: Start and attach to a sandbox environment (`--detach`)

---

## MCP Servers, Skills, Templates & Volumes

### `sbx mcp`
Register and manage Model Context Protocol (MCP) servers for sandbox sessions.

- **Usage:** `sbx mcp COMMAND`
- **Subcommands:**
  - `sbx mcp add`: Register an MCP server
  - `sbx mcp auth`: Authorize MCP servers (`sbx mcp auth rm`, `sbx mcp auth status`)
  - `sbx mcp inspect`: Show MCP server details
  - `sbx mcp load`: Load an already-registered MCP server into a sandbox
  - `sbx mcp ls`: List MCP servers
  - `sbx mcp rm`: Remove a registered MCP server
  - `sbx mcp unload`: Unload an MCP server from a running sandbox

### `sbx skills` *(experimental)*
Manage shared agent skills linked into sandbox skill directories.

- **Usage:** `sbx skills COMMAND`
- **Subcommands:**
  - `sbx skills add`: Add skills from a Git repository
  - `sbx skills import`: Import skills from supported host agent directories
  - `sbx skills ls`: List installed skills
  - `sbx skills rm`: Remove installed skills
  - `sbx skills update`: Update skills added from repositories

### `sbx template`
Manage saved sandbox snapshot templates.

- **Usage:** `sbx template COMMAND`
- **Subcommands:**
  - `sbx template inspect`: Show template details
  - `sbx template load`: Load an image from a tar file into the sandbox runtime
  - `sbx template ls`: List template images
  - `sbx template rm`: Remove a template image
  - `sbx template save`: Save a snapshot of a sandbox as a template

### `sbx volume` *(cloud-only)*
Manage persistent storage volumes that survive across cloud sandbox runs (requires `--cloud`).

- **Usage:** `sbx volume COMMAND`
- **Subcommands:**
  - `sbx volume create`: Create a new persistent volume
  - `sbx volume inspect`: Show volume details
  - `sbx volume ls`: List persistent volumes
  - `sbx volume rm`: Remove a persistent volume

---

## Scheduled Runs (`sbx schedule`) *(cloud-only)*

Manage schedules that start cloud sandboxes on a cron cadence (all subcommands require `--cloud`).

- **Usage:** `sbx schedule COMMAND`
- **Subcommands:**
  - `sbx schedule create`: Create a schedule that runs a kit on a cron cadence
  - `sbx schedule inspect`: Show schedule details
  - `sbx schedule ls`: List schedules
  - `sbx schedule rm`: Remove a schedule
  - `sbx schedule run`: Start a run of a schedule now
  - `sbx schedule runs`: List the runs of a schedule (`sbx schedule runs inspect`)
  - `sbx schedule update`: Update a schedule's settings or what it runs

---

## Daemon, Diagnostics & Utilities

### `sbx daemon`
Manage the local `sandboxd` background daemon.

- **Usage:** `sbx daemon COMMAND`
- **Subcommands:**
  - `sbx daemon log-level`: Show `sandboxd` per-category log levels (`sbx daemon log-level set`)
  - `sbx daemon restart`: Restart the `sandboxd` daemon
  - `sbx daemon start`: Start the `sandboxd` daemon
  - `sbx daemon status`: Show `sandboxd` daemon status
  - `sbx daemon stop`: Stop the `sandboxd` daemon

### `sbx df`
Show runtime disk usage for local images, containers, and managed volumes *(local-only)*.

- **Usage:** `sbx df [flags]`
- **Options:**
  - `--format`: Output format: `table`, `json`, or `yaml`
  - `--json`: Output in JSON format
  - `-v, --verbose`: Show individual images, containers, and volumes

### `sbx diagnose`
Diagnose common issues with your `sbx` installation.

- **Usage:** `sbx diagnose [flags]`
- **Options:**
  - `--format`: Output format: `table`, `json`, `yaml`, or `github-issue`
  - `--json`: Output in JSON format
  - `--upload`: Upload diagnostics to Docker support (table format only)

### `sbx reset`
Reset Docker Sandboxes to a freshly-installed state (stops sandboxes, clears caches/registries/state/policies/SSH config/secrets, signs out, stops daemon).

- **Usage:** `sbx reset [flags]`
- **Options:**
  - `-f, --force`: Skip confirmation prompt
  - `--preserve-secrets`: Keep stored secrets

### `sbx completion`
Generate shell autocompletion scripts.

- **Usage:** `sbx completion [bash|fish|powershell|zsh]`

### `sbx version`
Show Docker Sandboxes version information.

- **Usage:** `sbx version [flags]`
- **Options:**
  - `--format`: Output format: `table`, `json`, or `yaml`
  - `--json`: Output in JSON format

---

## YAML File Reference

Docker Sandboxes uses two distinct declarative YAML schemas:
1. **Sandbox Environment Files (`sbxenv.yaml`)** (`schemaVersion: "1"`): Consumed by `sbx env` to configure a sandbox's agent, kits, workspace mounts, ports, environment variables, secrets, MCP servers, host lifecycle hooks, and VM/container resources.
2. **Kit Descriptors (`<kit>.yaml` / `spec.yaml`)** (`schemaVersion: "3"`): Consumed by BuildKit (`# syntax=docker/sandbox-kit:3`) and `sbx kit` to package agent workloads, tool mixins, or kit sets as OCI artifacts.

> **Important:** Swapping `schemaVersion` values between `sbxenv.yaml` (`"1"`) and Kit v3 descriptors (`"3"`) fails strict validation. Both parsers reject unrecognized keys.

---

### 1. Sandbox Environment File (`sbxenv.yaml`)

#### File Resolution & Merge Semantics
- **Default lookup (`sbx env run` without `PATH`):** Loads `~/.sbxenv.yaml` (user defaults, if present) as the base layer, then merges `./sbxenv.yaml` from the current working directory.
- **Explicit `PATH` (`sbx env run ./custom.yaml` or `sbx env run base.sbxenv.yaml local.sbxenv.yaml`):** Skips `~/.sbxenv.yaml` and deep-merges the specified files in declaration order (`docker compose -f` semantics). Passing a directory resolves to `<DIR>/sbxenv.yaml`.
- **Merge rules:**
  - Scalar fields: Later files override earlier values.
  - Maps (`env`, `args`, `secrets`, `bindings`, `registries`, `sandboxOptions`): Merge key-by-key; later keys override earlier ones.
  - `entrypoint` and `cmd`: Later arrays replace earlier arrays completely.
  - `ports`: Deduplicated by `(container, protocol)`; later matching entry replaces the entire earlier entry in-place.
  - `mcp.servers`: Deduplicated by exact `name`; later matching entry replaces the entire earlier entry in-place.
  - `additionalWorkspaces`: Deduplicated by container `target` (or resolved host `path` when `target` is omitted); later matching entry replaces the entire earlier entry in-place.
  - `kits`: Entries with the same `source` merge their `args` map by key.
  - Other lists: Concatenate in declaration order.
- **Variable interpolation (`${{ ... }}`):**
  - `${{ env.projectDir }}`: Directory from which `sbx env` was invoked (required root for `workspace` in `~/.sbxenv.yaml`).
  - `${{ env.fileDir }}`: Absolute path to the directory containing the declaring environment file.
  - `${{ args.<NAME> }}`: Value of an environment argument declared under `args:` (overridable via `--env-arg NAME=VALUE` or `--env-args-file`).
  - *Note:* Interpolation cannot be used in YAML keys or inside the `args:` block. Unquoted references are parsed as typed YAML values (e.g., integers/booleans); quote them to preserve strings.

#### Top-Level Fields (`schemaVersion: "1"`)

| Field | Type | Required | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `schemaVersion` | string | **Yes** | --- | Must be `"1"` |
| `agent` | string | **Yes** | --- | Built-in agent name (`claude`, `codex`, `cursor`, `devin`, `docker-agent`, `gemini`, `opencode`, `shell`) or manifest name of a workload kit in `kits:` |
| `name` | string | No | Derived from directory | Sandbox name (overridable via `--name`; disallowed in `~/.sbxenv.yaml`) |
| `entrypoint` | list of strings | No | Image/kit default | Override container launch entrypoint (exec argv, not shell-expanded) |
| `cmd` | list of strings | No | Image/kit default | Override default launch arguments appended to `entrypoint` (replaced by CLI args after `--`) |
| `args` | map | No | `{}` | Environment template arguments (`<NAME>: { default: <VAL>, description: <STR> }`) referenced via `${{ args.<NAME> }}` |
| `kits` | list | No | `[]` | Workload or mixin kits to apply (strings or objects with `source` and `args`) |
| `workspace` | string or object | No | None | Primary host directory mount (local-only; unsupported with `--cloud`) |
| `additionalWorkspaces` | list | No | `[]` | Extra host directory mounts (local-only; unsupported with `--cloud`) |
| `ports` | list | No | `[]` | Sandbox ports to publish to the host (local-only; unsupported with `--cloud`) |
| `env` | map | No | `{}` | Key-value environment variables injected into the sandbox |
| `secrets` | map | No | `{}` | Sandbox-scoped service credentials provisioned on creation |
| `bindings` | map | No | `{}` | Host-global credential domain bindings merged into `~/.config/sbx/credentials.yaml` |
| `registries` | map | No | `{}` | Sandbox-scoped private container registry credentials (local-only) |
| `mcp` | object | No | `{}` | MCP server registrations (`servers:` list; local-only) |
| `lifecycle` | object | No | `{}` | Host-executed lifecycle hooks (`initialize`, `postCreate`) |
| `sandboxOptions` | object | No | `{}` | VM/container sizing, template, and runtime feature flags |

#### Nested Field Specifications

**`kits` (list items)**
Can be a string reference (local path prefixed with `./` or `../`, Git URL, ZIP archive, or OCI image reference) or an object:
- `source` (string, **required**): Kit path, URL, or OCI image reference.
- `args` (map, optional): Key-value arguments passed to the kit (overridable via `--kit-arg` / `--kit-args-file`).

**`workspace` & `additionalWorkspaces[]`** *(local-only)*
- String shorthand (for `workspace` or `additionalWorkspaces[]`): Host path resolved relative to the declaring file's directory.
- Object form:
  - `path` (string, **required**): Host directory path.
  - `target` (string, optional --- `additionalWorkspaces` only): Custom mount destination path inside the sandbox.
  - `readOnly` (bool, optional, default `false`): Mount directory read-only inside the sandbox.
  - `clone` (bool, optional): Clone the repository instead of bind-mounting directly.

**`ports[]`** *(local-only)*
- `container` (int, **required**): Sandbox port (`1`--`65535`).
- `host` (int, optional): Host port to bind (omitting allocates an ephemeral host port).
- `hostIP` (string, optional): Host IP interface to bind (e.g., `127.0.0.1`).
- `protocol` (string, optional): `tcp`, `udp`, `tcp4`, or `tcp6` (case-insensitive; defaults to `tcp4`, or `tcp6` for IPv6 `hostIP`).

**`secrets`, `bindings`, and `registries`**
- `secrets.<SERVICE_NAME>`: Provisions sandbox-scoped credentials from host environment variables, files, or host helper commands (supports `snapshot: true` for `--cloud` to resolve once at creation).
- `bindings.<SERVICE_NAME>`: Approves/merges per-service domain routing rules into `~/.config/sbx/credentials.yaml` (persists across `sbx env rm` unless `--prune-bindings` is passed).
- `registries.<REGISTRY_HOST>` *(local-only)*: Provisions sandbox-scoped registry authentication for pulling images inside the sandbox.

**`mcp`** *(local-only)*
- `mcp.servers[]`:
  - `name` (string, **required**): Unique server identifier.
  - `command` (string or list, optional): Host command for local stdio MCP servers (scoped to the environment).
  - `args` (list of strings, optional): Arguments for `command`.
  - `env` (map, optional): Environment variables for local MCP process.
  - `url` (string, optional): Endpoint URL for remote HTTP/SSE MCP servers (registered host-globally).

**`lifecycle`** *(executes on the host with user permissions; skipped with `--skip-host-commands`)*
- `initialize[]`: Runs on every `sbx env create` and `sbx env run` (including reattaching to an existing sandbox).
- `postCreate[]`: Runs once immediately after sandbox creation and before interactive attach (receives `SBX_SANDBOX_NAME`).
  - Entry fields: `name` (string, optional display label), `command` (string, **required**).

**`sandboxOptions`**

| Field | Type | Cloud Support | Description |
| :--- | :--- | :--- | :--- |
| `cpus` | int/number | Yes | vCPU allocation for the sandbox |
| `memory` | string | Yes | Memory limit (e.g., `4g`, `8g`, `12g`; up to host total with warning above 75%) |
| `rootSize` | string | Yes | Root filesystem disk size |
| `dockerSize` | string | Yes | Internal `/var/lib/docker` storage disk size |
| `clonedWorkspaceSize` | string | No | Disk allocation for cloned workspaces |
| `template` | string | No | Custom sandbox template image reference |
| `profile` | string | No | Sandbox runtime profile |
| `pullPolicy` | string | No | Image pull policy (`always`, `missing`, `never`) |
| `skills` | string | No | Shared skills mount mode (`readonly`, `readwrite`, `off`) |
| `fuse` | bool | No | Enable FUSE filesystem support inside local sandboxes |
| `gpu` | bool/config | No | Enable GPU passthrough |
| `usb` | bool/config | No | Enable USB passthrough |
| `display` | bool/config | No | Enable graphical display forwarding |
| `writableEnvFiles` | bool | No | Allow the sandbox agent to modify `sbxenv.yaml` (default `false`; mounted read-only by default) |

#### Complete `sbxenv.yaml` Example
```yaml
schemaVersion: "1"
name: web-platform
agent: codex
workspace: ./web-app
additionalWorkspaces:
  - path: ./shared-components
  - path: ./architecture-docs
    readOnly: true

args:
  nodeEnv:
    default: development

env:
  NODE_ENV: ${{ args.nodeEnv }}
  GOFLAGS: -mod=readonly

kits:
  - source: docker.io/sbx/playwright:1.61.1
  - source: ./kits/custom-linter
    args:
      strict: "true"

ports:
  - container: 3000
    host: 3000
    protocol: tcp

bindings:
  github:
    domains:
      - api.github.com
      - github.com

mcp:
  servers:
    - name: local-docs
      command: npx
      args: ["-y", "@modelcontextprotocol/server-filesystem", "./architecture-docs"]

lifecycle:
  postCreate:
    - name: Verify sandbox health
      command: sbx exec "$SBX_SANDBOX_NAME" node --version

sandboxOptions:
  cpus: 4
  memory: 8g
  fuse: true
  writableEnvFiles: false
```

---

### 2. Kit v3 Descriptor YAML (`<kit>.yaml`)

Kit v3 descriptors are compiled into OCI images using BuildKit (`# syntax=docker/sandbox-kit:3`). A kit can be structured as:
- **Companion pair:** `<name>.yaml` + `<name>.dockerfile` (matched by filename stem).
- **Single-file inline:** `<name>.yaml` containing an inline `build:` block.
- **Comment descriptor:** A Dockerfile embedding YAML declarations inside a `# kit:` comment block.
- **Kit set (`kind: set`):** A YAML descriptor composing multiple published kits via `kits:` without a Dockerfile.

#### Top-Level Descriptor Fields (`schemaVersion: "3"`)

```yaml
# syntax=docker/sandbox-kit:3
# yaml-language-server: $schema=https://raw.githubusercontent.com/docker/sandbox-kit-spec/main/schema/kit.schema.json
```

| Field | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `schemaVersion` | string | **Yes** | Must be `"3"` |
| `kind` | string | **Yes** | Kit role: `workload` (base agent/shell environment), `mixin` (tool/policy overlay), or `set` (curated composition of kits) |
| `displayName` | string | Recommended | Human-readable kit title |
| `description` | string | Recommended | Summary of what the kit provides |
| `version` | string | Recommended | Semantic version string (required for publishing) |
| `author` | string | No | Author or organization name |
| `sourceUrl` | string | Recommended | Upstream source repository URL |
| `iconUrl` | string | No | URL to kit icon asset |
| `licenses` | list of strings | Recommended | SPDX license identifiers (e.g., `[MIT]`, `[Apache-2.0]`) |
| `provides` | list of strings | No | Matchable virtual capability/package identifiers (e.g., `["gh@2.72.0"]`) |
| `requires` | list of strings | No | Required virtual identifiers that must be satisfied by the composed kit set |
| `conflicts` | list of strings | No | Virtual identifiers that cannot coexist with this kit |
| `args` | map | No | Build-time and create-time typed arguments exposed as `${{ kit.args.<NAME> }}` |
| `build` | string/object | Conditional | Inline Dockerfile content (for single-file kits without a companion `.dockerfile`) |
| `kits` | list | Conditional | Component kits to compose (required when `kind: set`; entries take `ref:` and optional `args:`) |
| `capabilities` | list | No | Typed, versioned host resource grants and runtime behaviors |

#### Well-Known Capabilities (`capabilities[]`)
Each item in `capabilities` specifies `type` (required), `optional` (bool, default `false`), `description` (string, optional), and capability-specific `config`:

| Capability Type | Shape | Permission Gate | Purpose & Key `config` Fields |
| :--- | :--- | :--- | :--- |
| `com.docker.sandbox/sbx@1` | Singleton | No | Declares workload compliance for `sbx` (no `config`; requires `/bin/sh`, `/bin/bash`, non-empty image `USER` in `/etc/passwd`, and `BASH_ENV`) |
| `com.docker.sandbox/lifecycle@1` | Singleton | No | `install[]` (once at create: `command`, `user`, `env[]`, `description`), `startup[]` (every boot: `command`, `user`, `background`, `env[]`), `files[]` (`path`, `content`, `mode`, `overwrite`), and `interactive` (argv tail for TTY sessions) |
| `com.docker.sandbox/network-policy@1` | Instance | Yes | Outbound network allowlists split by phase (`install.allow[]`, `runtime.allow[]`) |
| `com.docker.sandbox/credential@1` | Instance | Yes | Proxy-injected service credentials (`service`, `phase`: `install`\|`runtime`, `inject[]` domains --- every injected domain must be allowed in that phase's `network-policy@1`) |
| `com.docker.sandbox/ssh-agent@1` | Singleton | Yes | Requests host SSH agent socket forwarding into the sandbox |
| `com.docker.sandbox/port@1` | Instance | Yes | Declares inbound container port to publish (`name`, `container`: `1..65535`, `transport`: `tcp`\|`udp`, `family`: `ipv4`\|`ipv6`\|`dual`) |
| `com.docker.sandbox/agent-context@2` | Instance | No | Injects inline agent guidance (`content`, optional `directory` and `filename`; replaces `@1` `contentFile`) |
| `com.docker.sandbox/agent-interactive-sessions@1` | Singleton | No | Maps interactive TUI CLI verbs (`prompt`, `resume`, `continue`, `newSession`, `sessionPicker`) |
| `com.docker.sandbox/agent-sessions@1` | Singleton | No | Maps headless/programmatic session CLI verbs |
| `com.docker.sandbox/agent-skills@1` | Singleton | No | Declares support for mounting host shared skills |
| `com.docker.sandbox/long-running@1` | Singleton | No | Prevents session-idle auto-stop when all clients disconnect (no `config`) |
| `com.docker.sandbox/dockerd@1` | Singleton | Yes | Provisions private persistent `/var/lib/docker` storage and starts `dockerd` inside the sandbox |
| `com.docker.sandbox/kit-registry@1` | Singleton | Yes | Routes sandbox access to the runtime's local Kit registry endpoint (no `config`) |

#### Complete Kit v3 Mixin Example (`gh.yaml`)
```yaml
# syntax=docker/sandbox-kit:3
schemaVersion: "3"
kind: mixin
displayName: GitHub CLI
description: gh CLI overlay with GitHub API credential injection
sourceUrl: https://github.com/cli/cli
version: "2.72.0"
licenses: [MIT]
provides: ["gh@2.72.0"]

capabilities:
  - type: com.docker.sandbox/network-policy@1
    config:
      runtime:
        allow:
          - api.github.com
          - github.com

  - type: com.docker.sandbox/credential@1
    optional: true
    description: GitHub API access for gh
    config:
      service: github
      phase: runtime
      inject:
        - domain: api.github.com
        - domain: github.com

  - type: com.docker.sandbox/agent-context@2
    config:
      content: |
        Use `gh` for GitHub pull requests, issues, and repository queries.
```
