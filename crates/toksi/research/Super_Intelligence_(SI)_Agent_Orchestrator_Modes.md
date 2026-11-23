# Super Intelligence (SI) Agent Orchestrator Modes

## 1. Meta-Cognition & Governance (The "Self")
- **Orchestrate / Route**: Evaluates incoming goals, decomposes complex objectives into sub-tasks, assigns modes/sub-agents, and manages execution priority.
- **Reflect / Self-Critique**: Evaluates the agent's own reasoning, outputs, and failures to detect hallucinations, logical fallacies, or inefficient paths before acting.
- **Align / Guardrail**: Verifies that proposed actions comply with safety boundaries, ethical constraints, resource limits, and user intent before execution.
- **Evolve / Self-Improve**: Updates system prompts, refines routing heuristics, and creates reusable skills/tools based on past task performance.

## 2. Memory & World Modeling
- **Recall / Retrieve**: Queries vector databases, knowledge graphs, and episodic memory to pull relevant past experiences, user preferences, and context.
- **Consolidate / Memorize**: Compresses completed sessions into long-term structured memory, updating internal world models and pruning obsolete data.
- **Simulate / Predict**: Runs mental sandboxes or Monte Carlo thought trees to forecast second- and third-order consequences of a plan before committing resources.

## 3. Discovery & Epistemics
- **Chat / Interface**: Natural language communication layer for negotiating goals, asking clarifying questions, and delivering human-readable updates.
- **Brainstorm / Diverge**: Generates novel hypotheses, unconventional solutions, and lateral connections across unrelated domains.
- **Research / Ground**: Autonomously browses, queries APIs, cross-references primary sources, and assigns confidence scores to external facts.
- **Synthesize / Abstract**: Distills massive multi-modal datasets, codebases, or research papers into core first principles and actionable mental models.

## 4. Pedagogy & Knowledge Transfer
- **Study / Deconstruct**: Breaks down unfamiliar domains into foundational axioms to build rapid mastery (for either the user or the agent itself).
- **Coach / Socratic**: Guides human collaborators through complex problem-solving using targeted questions, mental model checks, and adaptive challenges.

## 5. Strategy & Architecture
- **Plan / Decompose**: Constructs Directed Acyclic Graphs (DAGs) of dependencies, milestones, fallback paths, and resource budgets for multi-step goals.
- **Toolsmith / Architect**: Designs, writes, and registers new custom tools, API wrappers, or micro-agents when existing capabilities cannot solve a task.

## 6. Autonomous Execution
- **Write Code / Build**: Generates production-grade software, scripts, and system integrations following strict architectural specs.
- **Refactor / Optimize**: Re-engineers existing codebases or workflows for maximum algorithmic efficiency, minimal token/compute cost, and maintainability.
- **Act / Operate**: Interacts directly with external environments (shell, browser, APIs, file systems, cloud infrastructure) to execute real-world state changes.

## 7. Verification & Resilience
- **Running Tests / Verify**: Generates adversarial test suites, benchmarks outputs against formal specifications, and validates edge cases.
- **Debug / Heal**: Autonomously reads stack traces, forms hypotheses, isolates root causes, patches failures, and re-runs verification loops until green.
- **Audit / Red-Team**: Actively attempts to break its own plans, exploit its own code, and uncover blind spots or security vulnerabilities.
- **Document / Log**: Maintains real-time telemetry, architectural decision records (ADRs), and clean human-readable documentation of all autonomous actions.

---

## Autonomous Loop Example (How an SI Agent Processes a Complex Goal)

| Step | Active Mode | Internal Agent Action |
| :--- | :--- | :--- |
| **1** | **Recall / Retrieve** | Checks memory for existing architecture, constraints, and past similar tasks. |
| **2** | **Research / Ground** | Gathers up-to-date specs, libraries, or domain data to fill knowledge gaps. |
| **3** | **Simulate / Predict** | Models 3 different architectural approaches and forecasts bottlenecks. |
| **4** | **Plan / Decompose** | Builds a step-by-step execution graph with validation checkpoints. |
| **5** | **Audit / Red-Team** | Critiques the plan for security risks or flawed assumptions before writing code. |
| **6** | **Write Code / Build** | Implements the system modularly. |
| **7** | **Running Tests / Verify** | Executes unit, integration, and adversarial edge-case tests. |
| **8** | **Debug / Heal** | Automatically catches a failing test, traces the bug, and patches it. |
| **9** | **Consolidate / Memorize** | Saves the newly learned solution pattern to long-term memory for future tasks. |
