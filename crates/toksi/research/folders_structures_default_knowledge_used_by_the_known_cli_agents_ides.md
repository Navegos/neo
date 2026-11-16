#This folders structures are the default knowledge for the folder structure used by the known cli/agents/ides.

.agents Protocol
An open directory convention for AI agent configuration. One place for MCP tools, AGENTS.md instructions, skills, Sub-Agents, Tasks, Memories, and model config — in plain, version-controllable files.

	Open standard agent's folder, The .agents/ folder is a standardized, vendor-agnostic directory convention used to store AI agent configurations, guidelines, skills, and memory in a project.

	~/.agents/
	directory layout
	$ tree .agents/

	.agents/
	├── speakmcp-settings.json  	# general settings
	├── mcp.json                	# MCP servers & tool connector definitions
	├── models.json             	# Model presets, routing configurations, & API keys
	├── system-prompt.md        	# Global baseline persona or system instruction override
	├── agents.md               	# Core project instructions & style guides (AGENTS.md)
	├── dotagents-settings.json     # Protocol runner configuration & engine preferences
	├── layouts/
	│   └── ui.json                 # UI, terminal, & canvas layout layout preferences
	├── agents/						# Multi-Agent ecosystem definitions (ACP compatible)
	│   ├── <agent-id>/
	│   │   ├── agent.md         	# Sub-agent profile & local system prompt (with Frontmatter)
	│   │   └── config.json      	# Agent-specific tool permissions & routing limits
	│   └── deploy-specialist/
	│       └── agent.md
	├── skills/						# Composable, codified procedural workflows
	│   ├── <skill-id>/
	│   │   └── skill.md			# Execution recipe, rules, & step-by-step logic
	│   └── deploy-pipeline/
	│       └── skill.md
	├── tasks/                      # Automations, loops, & scheduled execution contexts
	│   └── <task-id>/
	│       └── task.md             # Declarative definition & loop triggers (e.g. cron interval)
	├── knowledge/                  # Persistent semantic memory & multi-modal documentation
	│   └── <knowledge-domain>/
	│       ├── architecture.md     # Markdown-based vector context injection files
	│       ├── diagram.png         # Contextual domain asset (image/diagram support)
	│       └── db-schema.pdf       # Dense technical documentation payload
	└── memories/          			# Persistent memory notes
	│   ├── arch-decisions.md
	│   └── user-prefs.md
	└── .backups/               	# auto-rotated, timestamped system snapshots
		├── skills/
		└── memories/

	In this folders we only save and use Open agent's standards compatibles.

	OpenAgents Global shared path: (user/).agents/
	OpenAgents Workspace|repository project path: (repository workspace dir/).agents/

	agents/ mcp.json, models.json, system-prompt.md, agents.md, dotagents-settings.json, agents/<name>.md(and config.json) or agents/<name>/agent.md(and config.json), skills/<name>/<skill>.md, tasks/<name>/<task>.md, knowledge/<architecture>.md, memories/<name>.md, .backups/, (mcp/ folder with (mcp/codegraph/, mcp/context/ mcp/projectmem/ folders <etc's>) converted to skills/toolname example(skills/codegraph))

	in repository/ AGENTS.md CLAUDE.md GEMINI.md <etc's>.

	Everithing else not compatible with Open agent's standards, and or (agent vendor provider) goes to the correct (agent vendor provider folder structure).

.Antigravity protocol
	Antigravity Global shared path: (user/).gemini/
	Antigravity Workspace|repository project path: (repository workspace dir/).gemini/

	The Antigravity APP settings, agy cli settings, Antigravity IDE settings, we have:
	.gemini/ hooks.json, mcp_config.json, settings.json, rules/<name>.md or rules/<name>/<name>.md, GEMINI.md, rules.json, skills.json, agents/<name>.md or agents/<name>/<agent>.md, skills/<name>/<SKILL>.md, plugins/<plugin-folder>/, (mcp/ folder with (mcp/codegraph/, mcp/context/ mcp/projectmem/ folders <etc's>) converted to skills/toolname example(skills/codegraph))
	.gemini/config/ config.json, hooks.json, mcp_config.json, settings.json, rules/<name>.md or rules/<name>/<name>.md, GEMINI.md, rules.json, skills.json, agents/<name>.md or agents/<name>/<agent>.md, skills/<name>/<SKILL>.md, plugins/<plugin-folder>/, (mcp/ folder with (mcp/codegraph/, mcp/context/ mcp/projectmem/ folders <etc's>) converted to skills/toolname example(skills/codegraph))
	.gemini/antigravity/ hooks.json, mcp_config.json, settings.json, rules/<name>.md or rules/<name>/<name>.md, GEMINI.md, rules.json, skills.json, agents/<name>.md or agents/<name>/<agent>.md, skills/<name>/<SKILL>.md, plugins/<plugin-folder>/, (mcp/ folder with (mcp/codegraph/, mcp/context/ mcp/projectmem/ folders <etc's>) converted to skills/toolname example(skills/codegraph))
	.gemini/antigravity-cli/ hooks.json, mcp_config.json, settings.json, rules/<name>.md or rules/<name>/<name>.md, GEMINI.md, rules.json, skills.json, agents/<name>.md or agents/<name>/<agent>.md, skills/<name>/<SKILL>.md, plugins/<plugin-folder>/, (mcp/ folder with (mcp/codegraph/, mcp/context/ mcp/projectmem/ folders <etc's>) converted to skills/toolname example(skills/codegraph))
	.gemini/antigravity-ide/ hooks.json, mcp_config.json, settings.json, rules/<name>.md or rules/<name>/<name>.md, GEMINI.md, rules.json, skills.json, agents/<name>.md or agents/<name>/<agent>.md, skills/<name>/<SKILL>.md, plugins/<plugin-folder>/, (mcp/ folder with (mcp/codegraph/, mcp/context/ mcp/projectmem/ folders <etc's>) converted to skills/toolname example(skills/codegraph))

.Copilot protocol

Copilot Global user and Workspace|repository project path: (user/).copilot/ and (repository workspace dir/).copilot/
	.copilot/
	├── agents/                 # Personal custom agents
	├── extensions/             # Personal extensions
	├── hooks/                  # User-level hooks
	├── instructions/           # Personal *.instructions.md files
	├── skills/                 # Personal skills
	├── copilot-instructions.md # Instructions for all sessions
	├── mcp-config.json         # User-level MCP servers
	├── lsp-config.json         # User-level LSP servers
	├── providers.json          # BYOK providers and models
	├── settings.json           # Personal CLI settings
	├── permissions-config.json  # Saved permissions
	├── logs/
	├── session-state/
	└── config.json              # Automatically managed application state

Github Global user and Workspace|repository project path: (user/).github/ and (repository workspace dir/).github/
	.github/
	├── agents/
	│   └── reviewer.agent.md
	├── hooks/
	│   └── ...
	├── instructions/
	│   └── frontend.instructions.md
	├── copilot-instructions.md
	├── copilot/
	│   ├── settings.json
	│   └── settings.local.json
	├── mcp.json
	│── skills/
	│	└── my-skill/
	│       └── SKILL.md

.Claude protocol

Claude Global user and Workspace|repository project path: (user/).claude/ and (repository workspace dir/).claude/

	.claude/ (or ~/.claude/)
	├── CLAUDE.md                   # Core project or global instructions loaded every session
	├── settings.json               # Shared configuration: permissions, hooks, env vars, model defaults
	├── settings.local.json         # Personal local overrides (gitignored)
	├── .mcp.json                   # Team-shared Model Context Protocol (MCP) servers
	├── .worktreeinclude            # Files to carry over into new git worktrees
	├── rules/                      # Topic-scoped or path-gated markdown rules
	│   ├── testing.md
	│   └── security-design.md
	├── skills/                     # Reusable workflows/prompts invoked via /name or auto-invoked
	│   └── <skill-name>/
	│       └── SKILL.md
	├── commands/                   # Single-file custom slash commands (.md)
	│   └── <command-name>.md
	├── agents/                     # Specialized AI subagent definitions (YAML/Markdown)
	│   └── code-reviewer.md
	├── workflows/                  # Multi-step automated agent procedures
	└── agent-memory/               # Persistent auto-memory storage
		└── <agent-name>/
			└── MEMORY.md

.Codex ChatGPT OpenAI protocol

Codex Global user and Workspace|repository project path: (user/).codex/ and (repository workspace dir/).codex/

	.codex/
	│
	├── config.toml
	├── AGENTS.md
	├── AGENTS.override.md
	├── hooks.json
	│
	├── agents/
	│   ├── architect.toml
    │   ├── backend.toml
    │   ├── frontend.toml
    │   ├── database.toml
	│   ├── reviewer.toml
	│   │── researcher.toml
    │   │── tester.toml
    │   ├── debugger.toml
    │   └── security.toml
	│
	├── rules/
    │   ├── default.rules
    │   ├── shell.rules
    │   ├── git.rules
    │   ├── testing.rules
    │   └── security.rules
	│
	├── auth.json
	├── history.jsonl
	│
	├── worktrees/
	├── plugins/
	└── logs/

.Cursor protocol

Cursor Global user and Workspace|repository project path: (user/).cursor/ and (repository workspace dir/).cursor/

	.cursor/
	├── rules/                    # Global AI rules applied across all projects
	│   ├── universal-styling.mdc # e.g., default formatting preferences
	│   └── coding-safety.mdc     # e.g., general security practices to enforce
	├── skills/                   # Global agent skills available on this machine
	│   └── custom-git-tool/      # A modular automation folder
	│       └── SKILL.md          # Standard instruction file for the skill agent
	├── agents/                   # Machine-wide custom agent blueprints (.md files)
	├── commands/                 # Global slash commands invoked via (⌘ ⇧ J) menu
	│   └── code-review.md        # File name maps directly to the command string
	├── hooks.json				  #
	├── cli-config.json           # Stores core CLI preferences and terminal sync rules
	└── mcp.json                  # Global Model Context Protocol integrations
	├── permissions.json		  #

.Grok protocol

Grok Global user and Workspace|repository project path: (user/).grok/ and (repository workspace dir/).grok/

	.grok/                            # override the root with GROK_HOME
	├── config.toml                   # personal defaults (full config)
	├── managed_config.toml           # per-user managed defaults (enterprise)
	├── requirements.toml             # user policy pins (cannot be overridden below)
	├── sandbox.toml                  # custom sandbox profiles
	├── trusted_folders.toml          # /hooks-trust and --trust decisions
	├── mcp_credentials.json          # OAuth tokens for remote MCP servers
	├── auth.json                     # session credentials (failed team-pin logins are not written)
	├── hooks-paths                   # extra hook roots
	├── known layout under plugins:
	│   plugins/
	│   ├── marketplaces/             # marketplace installs
	│   └── known_marketplaces.json   # marketplace sources (also [[marketplace.sources]] in config.toml)
	├── skills/                       # user skills (each skill is a folder with SKILL.md)
	├── plugins/                      # user plugins
	├── hooks/*.json                  # personal lifecycle hooks
	├── agents/                       # custom subagent types
	├── personas/*.toml               # persona overlays
	├── sessions/                     # conversations, keyed by working directory
	├── worktrees/<repo>/<name>/      # git worktrees for isolated sessions
	├── logs/mcp/<server>.stderr.log  # stdio MCP stderr
	└── crash/                        # panic reports when GROK_CRASH_HANDLER=1

We have some examples for agent's folders structures, mcp, hooks, settings configurations, .json, .toml, <etc's>, in the folder:
neo\inspect-vault\agents-structures


This lists provides hooks keywords for the tools hooking process in hooks settings files, research wath we can include for our tools, or its missing from the tools, type of completeness and compatibility with the tools we use.
================================================================================
CATEGORY 1: AGENTIC CLI & BRIDGE PROVIDERS
================================================================================

[GitHub Copilot CLI] (.github/hooks/*.json / CLI daemon)
  - sessionStart                : Fired upon agent session initialization/resume
  - sessionEnd                  : Fired upon session termination or exit
  - userPromptSubmitted         : Triggered when user submits a prompt before model evaluation
  - userPromptTransformed       : Fired when prompt expansion/internal rewrite occurs
  - preToolUse                  : Prior to invoking tools/commands (can block/deny execution)
  - postToolUse                 : Immediately after tool execution finishes
  - agentStop                   : Main agent finishes turn response
  - subagentStop                : Worker/subagent completes task before parent handover
  - preCompact                  : Triggered prior to conversation memory/context compaction
  - errorOccurred               : Fired upon tool runtime errors or execution faults

[Claude Code (Anthropic)] (.claude/settings.json / hooks.json)
  - SessionStart                : Session initialization or checkpoint restore
  - SessionEnd                  : Session termination and teardown
  - Setup                       : One-time setup via /setup, --init, or --maintenance
  - UserPromptSubmit            : Intercepts prompt submission (can mutate or reject)
  - UserPromptExpansion         : Evaluates custom command expansion before conversion to prompt
  - PreToolUse                  : Before executing any tool (supports allow/ask/modify/deny)
  - PostToolUse                 : Post-successful tool execution (linters, formatters)
  - PostToolUseFailure          : When a tool execution throws or exits non-zero
  - PostToolBatch               : Resolves combined results after parallel batch execution
  - PermissionRequest           : When prompt/tool triggers manual user approval
  - PermissionDenied            : Triggered when auto-mode or policy denies execution
  - SubagentStart               : Injects isolated context into newly spawned subagent
  - SubagentStop                : Validates delegated work; can reject stop to mandate more work
  - TaskCreated                 : TaskCreate tool adds task to local tracker
  - TaskCompleted               : Task marked finished; validates test/deliverable evidence
  - Stop                        : Final response turn check (can block completion if tests fail)
  - StopFailure                 : Turn ends prematurely from API/network/auth errors
  - TeammateIdle                : Teammate agent becomes idle in multi-agent team mode
  - InstructionsLoaded          : Fired when CLAUDE.md or rules are injected into context
  - ConfigChange                : When local or project configuration is modified
  - CwdChanged                  : Working directory changes during execution
  - DirectoryAdded              : Additional project directory/workspace added
  - FileChanged                 : Monitored file changes on disk
  - WorktreeCreate              : Isolated git worktree created for subagent/sandbox
  - WorktreeRemove              : Git worktree cleanup/removal
  - PreCompact                  : Triggered prior to context window compaction
  - PostCompact                 : Triggered after context compaction finishes
  - Elicitation                 : MCP server requests interactive user input
  - ElicitationResult           : Validates user input before returning it to the MCP server
  - Notification                : System desktop or external webhook alert dispatched
  - MessageDisplay              : Intercepts output rendering while assistant text displays

[OpenAI Codex CLI] (~/.codex/hooks.json / project config)
  - SessionStart                : CLI session bootstrap & environment validation
  - SessionEnd                  : CLI session close & log export
  - UserPromptSubmit            : Intercepts instruction before model parsing (allows rewrite)
  - PreToolUse                  : Pre-execution barrier for shell/fs tools (can deny/halt)
  - PostToolUse                 : Post-execution callback for formatters, test runners
  - PermissionRequest           : Intercepts elevated/sandbox breakout confirmation
  - PreCompact                  : Context window sliding compaction alert
  - PostCompact                 : Memory state verification post-truncation
  - Interrupt                   : User SIGINT/cancellation event
  - Stop                        : Turn resolution check (can block if criteria unmet)

[Gemini CLI / Antigravity CLI (Google)] (~/.gemini/hooks / antigravity.yaml)
  - SessionStart                : Agent workspace initialization & config binding
  - SessionEnd                  : Process wrap-up and sandbox artifact destruction
  - PreInvocation               : Intercepts prompt to inject dynamic RAG / GCP context
  - PostInvocation              : Intercepts raw token generation before tool dispatch
  - PreToolUse                  : Validates bash/fs execution; allows blocking and mutation
  - PostToolUse                 : Sanitizes and redacts sensitive data from tool output
  - PreCompact                  : Pre-summarization hook for state capture
  - PostCompact                 : Post-summarization hook for validation
  - Stop                        : Completion barrier (forces continuation if needed)

[Grok Build (xAI)] (~/.grok/hooks.json)
  - SessionStart                : Workspace agent boot
  - SessionEnd                  : Teardown & telemetry sync
  - UserPromptSubmit            : Prompt interception
  - PreToolUse                  : Pre-action boundary check (allow/deny decision payload)
  - PostToolUse                 : Post-action validation hook
  - PostToolUseFailure          : Tool execution failure trigger
  - PermissionDenied            : Automatic policy denial callback
  - SubagentStart               : Delegated worker initialization
  - SubagentStop                : Delegated worker completion check
  - PreCompact                  : Context compaction pre-hook
  - PostCompact                 : Context compaction post-hook
  - Stop                        : Turn completion verification (can block completion)
  - StopFailure                 : Turn aborted due to API failure
  - Notification                : System alerts and warnings

[Bifrost CLI (Maxim AI Bridge)] (.bifrost/hooks.yaml)
  - on_gateway_connect          : Fired when agent connects to proxy gateway
  - pre_route_request           : Intercepts inbound prompt before routing to provider
  - post_route_response         : Intercepts stream chunks/completions from provider
  - pre_mcp_forward             : Intercepts MCP tool calls routed through the bridge
  - post_mcp_forward            : Intercepts tool results returned across the bridge
  - on_gateway_disconnect       : Fired when gateway bridge shuts down

[Aider] (.aider.conf.yml / git-hooks)
  - pre-commit                  : Git commit hook running syntax check/linter before committing
  - post-commit                 : Post-commit verification hook
  - pre-run                     : Intercepts command execution prior to model inference
  - post-run                    : After LLM response and automated git edits applied
  - on-auto-test                : Fired when configured test suite runs on modified code

[Goose (Block / Linux Foundation)] (~/.config/goose/hooks / config.yaml)
  - session_created             : Fired when goose session starts
  - before_prompt_submit        : Intercepts user prompt before LLM dispatch
  - before_tool_call            : Tool execution gate (permits intercept/denial)
  - after_tool_call             : Tool execution observation and logging
  - on_error                    : Unhandled agent loop error hook
  - session_closed              : Cleanup and memory persist hook

[Kilo CLI] (.kilo/hooks.json)
  - on_init                     : Environment setup and rule ingestion
  - before_agent_step           : Intercepts single step iteration in agent loop
  - pre_command                 : Shell execution gate
  - post_command                : Shell execution analysis
  - on_finish                   : Task completion gate

[OpenCode] (.opencode/hooks.json)
  - session_start               : Agent environment initialization
  - session_shutdown            : Resource release and session archiving
  - before_agent_start          : Injects environment variables and dynamic prompts
  - turn_start                  : Single turn inference inception
  - turn_end                    : Single turn completion
  - tool_call                   : Synchronous hook to approve, block, or mutate tool args
  - tool_result                 : Output filtering before appending to history
  - shell.env                   : Intercepts environment variables before running terminal

[Qwen Code (Alibaba Cloud)] (~/.qwencode/hooks.json)
  - SessionInit                 : Workspace indexing and model connection
  - PreToolExecute              : Intercepts tool execution (permission check)
  - PostToolExecute             : Analyzes tool return values
  - PromptPreprocess            : Redaction and context attachment
  - TurnFinalize                : Completion confirmation

[Amazon Q Developer CLI] (~/.aws/amazon-q/hooks.json)
  - onSessionStart              : Session start and AWS credential check
  - onCommandBefore             : Intercepts CLI subcommands before execution
  - onCommandAfter              : Inspects CLI execution output
  - onToolInvocation            : Blocks or validates tool access
  - onSessionEnd                : Audit logging and teardown

[Warp AI / Agentic Terminal] (Warp Workflows / Rule Hooks)
  - onTerminalLaunch            : Terminal session initialization
  - onCommandSuggested          : Intercepts model suggested shell script
  - beforeCommandRun            : User execution approval trigger
  - onCommandFailure            : Automatic diagnosis hook triggered on non-zero exit
  - onContextCompaction         : Rolling buffer truncation hook

[Mentat] (.mentat/hooks.py)
  - on_session_start            : Initial context gathering hook
  - before_action               : Validates file edits/terminal commands
  - after_action                : Post-execution diff inspect hook
  - on_session_stop             : Session summary hook

[Plandex] (.plandex/hooks)
  - on_plan_created             : Intercepts newly synthesized multi-step plan
  - before_apply_change         : Pre-diff application confirmation
  - after_apply_change          : Post-diff application verification
  - on_revert                   : Triggered when a plan or change is rolled back

[Crush] (.crush/hooks.json)
  - on_session_boot             : Session initialization
  - pre_patch                   : Intercepts patch application to files
  - post_patch                  : Post-patch compilation/syntax verification
  - on_exit                     : Session wrap-up

[Devika] (devika/agent/hooks.py)
  - on_project_init             : Project workspace scaffolding hook
  - on_agent_state_change       : State machine transitions (Research, Code, Test)
  - before_browser_action       : Web browsing/crawling interception
  - before_shell_execution      : Subprocess bash execution gate
  - on_agent_completed          : Goal fulfillment hook

[OpenHands CLI / OpenDevin] (openhands/events/hooks.py)
  - ActionStartHook             : Intercepts actions (CmdRunAction, FileWriteAction, Browse)
  - ActionFinishedHook          : Post-action inspection and observation capture
  - StateChangeHook             : Agent lifecycle state shift (Idle, Running, Paused)
  - SecurityAnalyzerHook        : Blocks dangerous commands (sandboxing boundary)
  - ContextTruncationHook       : Triggered before context pruning

[SWE-agent] (sweagent/environment/hooks.py)
  - on_init                     : Docker container sandbox startup
  - on_setup_env                : Repository setup & test environment instantiation
  - on_action_start             : Pre-execution barrier for SWE shell tool
  - on_action_end               : Output analysis and observation mapping
  - on_turn_end                 : Turn evaluation hook
  - on_task_completed           : Patch evaluation against benchmark/test suite

[GPT-Engineer] (gpt_engineer/core/hooks.py)
  - pre_generate                : Intercepts specification/prompt before generation
  - post_generate               : Post-code generation syntax verification
  - pre_apply_code              : Pre-file write validation hook
  - post_apply_code             : Post-write build/execution verification

[Shell-Genie] (~/.config/shell-genie/hooks)
  - on_prompt                   : Prompt capture
  - pre_exec                    : Intercepts generated command before terminal invocation
  - post_exec                   : Captures return code of executed command


================================================================================
CATEGORY 2: AGENTIC LSP & PROTOCOL PROVIDERS
================================================================================

[Continue.dev (LSP Sidecar & Daemon)] (config.ts / config.yaml)
  - modifyConfig                : Runtime hook to mutate entire config, model, and slash commands
  - onDidReceiveMessage         : Intercepts incoming messages to sidecar
  - onDidSendMessage            : Intercepts outgoing messages to UI/IDE
  - beforeModelRequest          : Intercepts prompts, context items, and token limits
  - afterModelResponse          : Intercepts streaming tokens/completions
  - onContextLoad               : Fired when custom context providers resolve items
  - onAutocompleteTrigger       : Fired before inline completion dispatch
  - onAutocompleteAccept        : Fired when inline code insertion accepted
  - onSessionClose              : Teardown and telemetry dump

[Cline (MCP-Native Extension)] (cline_hooks / custom instructions)
  - onAgentModeSwitch           : Fired when switching modes (Code, Architect, Ask)
  - onPromptPrepared            : Injects dynamic rules before model invocation
  - preToolExecution            : Permission gate for file edits and bash execution
  - postToolExecution           : Post-action validation (triggers diagnostic sweep)
  - onMcpServerConnected        : Fired upon dynamic MCP server initialization
  - onMcpToolExecuted           : Audits MCP tool input/output data
  - onTaskCompleted             : Final review gate before declaring task done

[Roo Code (Roo Clinic)] (roo_hooks / settings.json)
  - onRoleChange                : Fired when active persona/mode changes
  - beforeToolRun               : Interception hook to block/approve file/terminal commands
  - afterToolRun                : Post-run observation and automatic linter pass
  - onCustomInstructionsLoad    : Dynamic system prompt enrichment
  - onCheckpointRestore         : Fired when reverting code/chat checkpoint
  - onTaskFinish                : Output validation gate

[lsp-ai (Language Server)] (lsp-ai.json)
  - onInitialize                : LSP client-server capabilities negotiation
  - textDocument/didOpen        : Document open event (triggers context cache)
  - textDocument/didChange      : Document buffer mutation (debounced completion hook)
  - preTransformCompletion      : Intercepts model completion before sending to client
  - postTransformCompletion     : Formats LSP completion items for client
  - textDocument/didSave        : Save hook triggering whole-file diagnostics/checks

[Tabby (Self-hosted Engine)] (config.toml / tabby-agent)
  - onAgentStartup              : Agent daemon connection
  - onDocumentOpen              : File context load
  - onCompletionRequest         : Inline completion pre-filter (AST analysis)
  - onCompletionDelivered       : Inline completion post-processing
  - onCompletionAccepted        : Acceptance telemetry event
  - onCompletionDismissed       : Rejection telemetry event

[Model Context Protocol (MCP Bridges & LSP Daemons)]
  - notifications/initialized   : Client confirms initialization
  - prompts/list_changed        : Informs client of dynamic prompt updates
  - resources/list_changed      : Dynamic resource index update
  - resources/subscribe         : Client subscribes to resource changes
  - resources/updated           : Emitted when resource contents mutate
  - tools/list_changed          : Informs client of toolset schema changes
  - roots/list_changed          : Workspace root URI mutation event
  - ping                        : Heartbeat verification event

[FauxPilot LSP]
  - on_init                     : LSP server startup
  - on_completion_query         : Queries local Triton/vLLM backend
  - on_filter_response          : Cleans repetitive tokens or malformed completions
  - on_disconnect               : Client connection terminated

[Sourcegraph Cody Agent / LSP Daemon]
  - initialized                 : Agent daemon readiness handshake
  - textDocument/didFocus       : Intercepts active buffer focus shifts
  - chat/submitMessage          : Intercepts chat query (attaches local codebase context)
  - edit/taskCreated            : Intercepts inline edit task creation
  - edit/taskApplied            : Intercepts inline edit diff application
  - autocomplete/execute        : In-line autocompletion hook
  - telemetry/recordEvent       : Observability event

[Supermaven Engine] (sm_agent daemon)
  - sm/init                     : Engine connection & authentication
  - sm/file_context_update      : File buffer state sync
  - sm/completion_request       : Fast completion generation trigger
  - sm/completion_accepted      : Inline snippet accepted
  - sm/state_reset              : Cache invalidation hook


================================================================================
CATEGORY 3: AGENTIC EDITORS & IDE PROVIDERS
================================================================================

[Cursor (Anysphere)] (.cursor/hooks.json / workspace config)
  - sessionStart                : Workspace session initialization/resume
  - sessionEnd                  : Workspace session closure
  - beforeSubmitPrompt          : Intercepts prompt submission (allows/cancels turn)
  - beforeReadFile              : File read access control (allow/deny)
  - beforeShellExecution        : Bash/terminal command gate (allow/ask/deny)
  - afterShellExecution         : Shell process termination & return code capture
  - beforeMCPExecution          : Pre-dispatch check for MCP tools (allow/ask/deny)
  - afterMCPExecution           : Intercepts and rewrites MCP response output
  - preToolUse                  : Generic tool invocation gate (mutate tool arguments)
  - postToolUse                 : Generic tool result hook (inject additional context)
  - afterFileEdit               : Triggered immediately when editor applies diff
  - subagentStart               : Background subagent task instantiation
  - subagentStop                : Subagent task termination verification
  - preCompact                  : Rolling context window compression hook
  - stop                        : Main agent loop termination check

[Windsurf / Cascade (Cognition / Codeium)] (.codeium/windsurf/hooks.json)
  - onCascadeStart              : Cascade agent conversation/task initialized
  - onCascadeEnd                : Cascade agent completes task
  - preActionExecute            : Intercepts action before execution (command, write, run)
  - postActionExecute           : Intercepts action output and validates workspace state
  - onCommandRun                : Specialized bash/terminal execution gate
  - onFileModify                : Fired when Cascade applies multi-file changes
  - onMCPToolCall               : Model Context Protocol invocation gate
  - onRuleLoad                  : Ingestion of .windsurfrules context
  - onContextExhausted          : Warning event when token threshold is reached

[GitHub Copilot Workspace & Agent Mode] (.github/copilot-agent-mode.json)
  - onSessionStart              : Agent session workspace binding
  - onUserPrompt                : Prompt submission interceptor
  - onPlanGenerated             : Intercepts proposed multi-step implementation plan
  - onPlanStepExecute           : Fires before each discrete step in the plan
  - onFileEditProposed          : Diff review gate before applying changes
  - onFileEditApplied           : Diff application confirmation
  - onTerminalCommand           : Terminal execution permission check
  - onAgentFinish               : Full plan execution completed

[Void Editor] (.void/hooks.json)
  - onEditorBoot                : Editor startup and local LLM connector initialization
  - beforePromptSend            : Pre-process user prompt & custom system prompt
  - beforeToolCall              : Intercepts local tool execution
  - afterToolCall               : Tool output inspection
  - onDiffApply                 : Inline diff acceptance event

[Zed (Zed AI / Assistant Mode)] (~/.config/zed/settings.json / slash commands)
  - assistant:before-query      : Intercepts prompt and context slash-command inclusions
  - assistant:after-query       : Stream processing and response formatting
  - slash-command:invoked       : Fired when custom slash command executes
  - context-provider:resolve    : Fired when dynamic context provider retrieves state
  - tool:before-execute         : Tool execution approval
  - tool:after-execute          : Tool result post-processing

[Trae (ByteDance)] (~/.trae/hooks.json)
  - onSessionOpen               : Trae workspace agent start
  - onUserQuery                 : Inbound query interception
  - beforeToolExecution         : Tool execution gate (allow/deny)
  - afterToolExecution          : Tool execution result formatting
  - onCodeApply                 : Code edit application hook
  - onSessionClose              : Workspace agent session termination

[Replit Agent] (.replit/agent_hooks.json)
  - onAgentBoot                 : Sandbox environment spinning up
  - onCheckpointCreate          : File tree checkpoint created before modifications
  - beforeShellCommand          : Terminal run approval gate
  - afterShellCommand           : Captures error output / exit codes
  - onPackageInstall            : Intercepts package manager installations (npm, pip, cargo)
  - onDeploymentTrigger         : Intercepts agent-triggered application deployment
  - onAgentHalted               : Agent finishes task or yields for user input

[Positron AI (Posit)] (~/.positron/agent/hooks)
  - onSessionStart              : Data science IDE agent startup
  - onPromptSubmit              : Intercepts natural language data query
  - beforeKernelExecution       : Intercepts R/Python code block before running in kernel
  - afterKernelExecution        : Inspects data frames, console output, plots
  - onVariableInspect           : Variable/environment inspection hook
  - onSessionEnd                : Session tear down

[PearAI] (.pearai/hooks.json)
  - onAgentInit                 : Agent initialization
  - beforeMessageSend           : Prompt formatting and inventory injection
  - preToolCall                 : Execution permission check
  - postToolCall                : Result formatting and linter invocation
  - onTaskComplete              : Task completion check

[Melty] (.melty/hooks.py)
  - on_boot                     : Boot hook; captures initial git state
  - before_commit_generation    : Evaluates diffs before committing
  - after_commit_generation     : Git commit creation validation
  - on_compiler_check           : Compiler error check hook
  - on_finish                   : Task termination

[Amazon Q Developer (JetBrains & VS Code)] (~/.aws/q-ide/hooks.json)
  - onAgentInit                 : IDE plugin agent bootstrap
  - onChatPromptSubmit          : Prompt interception and IAM permission validation
  - onCodeTransformStart        : Language/framework modernization plan start
  - onCodeTransformStep         : Transformation step execution
  - onSecurityScanStart         : Code vulnerability scan trigger
  - onSecurityScanComplete      : Vulnerability results inspection
  - onInlineAccepted            : Inline code suggestion accepted
  - onInlineRejected            : Inline code suggestion rejected
================================================================================
