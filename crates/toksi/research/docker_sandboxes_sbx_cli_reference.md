# `sbx` — Docker Sandboxes CLI Reference

**Description:** Manage AI coding agent sandboxes.  
**Usage:** `sbx COMMAND`

Docker Sandboxes creates isolated sandbox environments for AI agents, powered by Docker. Run agents locally via `sandboxd` or remotely in Docker Cloud Sandboxes using `--cloud`.

---

## Global Options

These flags apply to `sbx` and its subcommands:

| Option | Default | Description |
| :--- | :--- | :--- |
| `--cloud` | | Dispatch to Docker Cloud Sandboxes API instead of local `sandboxd` (supported by a growing set of verbs — run `sbx --cloud --help` for the current list) |
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
  - `sbx create claude` — Create a sandbox for Claude Code
  - `sbx create codex` — Create a sandbox for OpenAI Codex
  - `sbx create cursor` — Create a sandbox for Cursor
  - `sbx create devin` — Create a sandbox for Devin
  - `sbx create docker-agent` — Create a sandbox for Docker Agent
  - `sbx create gemini` — Create a sandbox for Gemini CLI
  - `sbx create opencode` — Create a sandbox for OpenCode
  - `sbx create shell` — Create a plain Bash shell sandbox without an agent
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
  - `--filter`: Filter candidates (supported: `until=TIMESTAMP` — RFC 3339, Unix timestamp, or Go duration like `168h`)
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