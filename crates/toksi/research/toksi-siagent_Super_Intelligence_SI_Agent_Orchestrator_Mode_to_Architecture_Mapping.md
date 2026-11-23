# `<toksi-siagent>` Orchestrator Mode-to-Architecture Mapping

## 1. Meta-Cognition & Governance (The "Self")
- **Orchestrate / Route**: Evaluates incoming goals, decomposes complex objectives into sub-tasks, assigns modes/sub-agents, and manages execution priority.
  - *Architecture*: `Orchestrator`, `LLM Router`, `Model Gateway`, `State Machine`, `Event Bus`, `Message Broker`
- **Reflect / Self-Critique**: Evaluates `<toksi-siagent>`'s own reasoning, outputs, and failures to detect hallucinations, logical fallacies, or inefficient paths before acting.
  - *Architecture*: `Verification Engine`, `Eval Harness`, `Tracing Engine`, `Telemetry Engine`
- **Align / Guardrail**: Verifies that proposed actions comply with safety boundaries, ethical constraints, resource limits, and user intent before execution.
  - *Architecture*: `Guardrails`, `Policy-as-Code Engine`, `IAM Attestation Layer`, `Circuit Breaker`, `Kill Switch`, `HITL Gateway`
- **Evolve / Self-Improve**: Updates system prompts, refines routing heuristics, and creates reusable skills/tools based on past task performance.
  - *Architecture*: `Eval Harness`, `Checkpointer`, `Immutable Audit Log`, `Tool Registry`

## 2. Memory & World Modeling
- **Recall / Retrieve**: Queries vector databases, knowledge graphs, and episodic memory to pull relevant past experiences, user preferences, and context.
  - *Architecture*: `Context Store`, `Episodic Memory`, `Semantic Memory`, `Vector Database`, `Knowledge Graph`, `Embedding Model`, `Reranker`, `Semantic Cache`
- **Consolidate / Memorize**: Compresses completed sessions into long-term structured memory, updating internal world models and pruning obsolete data.
  - *Architecture*: `Ingestion Pipeline`, `Episodic Memory`, `Semantic Memory`, `Knowledge Graph`, `Checkpointer`
- **Simulate / Predict**: Runs mental sandboxes or Monte Carlo thought trees to forecast second- and third-order consequences of a plan before committing resources.
  - *Architecture*: `World Model`, `Hierarchical Planner`, `Inference Engine`

## 3. Discovery & Epistemics
- **Chat / Interface**: Natural language communication layer for negotiating goals, asking clarifying questions, and delivering human-readable updates.
  - *Architecture*: `HITL Gateway`, `Context Store`, `LLM Router`, `Guardrails`
- **Brainstorm / Diverge**: Generates novel hypotheses, unconventional solutions, and lateral connections across unrelated domains.
  - *Architecture*: `World Model`, `Inference Engine`, `Knowledge Graph`
- **Research / Ground**: Autonomously browses, queries APIs, cross-references primary sources, and assigns confidence scores to external facts.
  - *Architecture*: `MCP Server`, `Tool Registry`, `Ingestion Pipeline`, `Reranker`, `Vector Database`
- **Synthesize / Abstract**: Distills massive multi-modal datasets, codebases, or research papers into core first principles and actionable mental models.
  - *Architecture*: `Semantic Memory`, `Knowledge Graph`, `Inference Engine`

## 4. Pedagogy & Knowledge Transfer
- **Study / Deconstruct**: Breaks down unfamiliar domains into foundational axioms to build rapid mastery (for either the user or `<toksi-siagent>` itself).
  - *Architecture*: `Semantic Memory`, `Knowledge Graph`, `Hierarchical Planner`
- **Coach / Socratic**: Guides human collaborators through complex problem-solving using targeted questions, mental model checks, and adaptive challenges.
  - *Architecture*: `Episodic Memory`, `Context Store`, `HITL Gateway`

## 5. Strategy & Architecture
- **Plan / Decompose**: Constructs Directed Acyclic Graphs (DAGs) of dependencies, milestones, fallback paths, and resource budgets for multi-step goals.
  - *Architecture*: `Hierarchical Planner`, `World Model`, `State Machine`, `Checkpointer`
- **Toolsmith / Architect**: Designs, writes, and registers new custom tools, API wrappers, or micro-agents when existing capabilities cannot solve a task.
  - *Architecture*: `Tool Registry`, `MCP Server`, `Code Sandbox`, `IAM Attestation Layer`

## 6. Autonomous Execution
- **Write Code / Build**: Generates production-grade software, scripts, and system integrations following strict architectural specs.
  - *Architecture*: `Code Sandbox`, `Inference Engine`, `Checkpointer`
- **Refactor / Optimize**: Re-engineers existing codebases or workflows for maximum algorithmic efficiency, minimal token/compute cost, and maintainability.
  - *Architecture*: `Code Sandbox`, `Telemetry Engine`, `Semantic Cache`
- **Act / Operate**: Interacts directly with external environments (shell, browser, APIs, file systems, cloud infrastructure) to execute real-world state changes.
  - *Architecture*: `Agent Runtime Daemon`, `MCP Server`, `Tool Registry`, `IAM Attestation Layer`, `Policy-as-Code Engine`

## 7. Verification & Resilience
- **Running Tests / Verify**: Generates adversarial test suites, benchmarks outputs against formal specifications, and validates edge cases.
  - *Architecture*: `Verification Engine`, `Code Sandbox`, `Eval Harness`
- **Debug / Heal**: Autonomously reads stack traces, forms hypotheses, isolates root causes, patches failures, and re-runs verification loops until green.
  - *Architecture*: `Tracing Engine`, `Telemetry Engine`, `Code Sandbox`, `Checkpointer`
- **Audit / Red-Team**: Actively attempts to break its own plans, exploit its own code, and uncover blind spots or security vulnerabilities.
  - *Architecture*: `Verification Engine`, `Policy-as-Code Engine`, `Immutable Audit Log`, `Circuit Breaker`
- **Document / Log**: Maintains real-time telemetry, architectural decision records (ADRs), and clean human-readable documentation of all autonomous actions.
  - *Architecture*: `Immutable Audit Log`, `Tracing Engine`, `Telemetry Engine`, `Ingestion Pipeline`

---

## `<toksi-siagent>` Autonomous Loop Example

| Step | Active Mode | Internal `<toksi-siagent>` Action |
| :--- | :--- | :--- |
| **1** | **Recall / Retrieve** | Queries `Vector Database`, `Knowledge Graph`, and `Episodic Memory` via `Reranker` into `Context Store`. |
| **2** | **Research / Ground** | Fetches up-to-date specs via `MCP Server` and `Tool Registry` through `Ingestion Pipeline`. |
| **3** | **Simulate / Predict** | Models 3 architectural approaches inside `World Model` via `Inference Engine` to forecast bottlenecks. |
| **4** | **Plan / Decompose** | `Hierarchical Planner` builds an execution DAG saved in `State Machine` and `Checkpointer`. |
| **5** | **Audit / Red-Team** | `Verification Engine` and `Policy-as-Code Engine` critique the plan, logging to `Immutable Audit Log`. |
| **6** | **Write Code / Build** | `Inference Engine` implements the system modularly inside `Code Sandbox`. |
| **7** | **Running Tests / Verify** | `Verification Engine` and `Eval Harness` execute unit, integration, and adversarial edge-case tests. |
| **8** | **Debug / Heal** | `Tracing Engine` and `Telemetry Engine` catch failing tests, trace bugs, and patch inside `Code Sandbox`. |
| **9** | **Consolidate / Memorize** | `Ingestion Pipeline` saves validated patterns to `Semantic Memory` and `Knowledge Graph`. |
