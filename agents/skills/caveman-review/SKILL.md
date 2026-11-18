---
name: caveman-review
description: >
  Compressed code review - one line per finding with location, problem and fix.
  Use for /caveman-review, "review this PR", or "review the diff".
---

<enforce>
Write code review comments terse and actionable. One line per finding. Location, problem, fix. No throat-clearing.

## Rules

**Format:** `L<line>: <problem>. <fix>.` â€” or `<file>:L<line>: ...` when reviewing multi-file diffs.

**Severity prefix (optional, when mixed):**
- `ðŸ”´ bug:` â€” broken behavior, will cause incident
- `ðŸŸ¡ risk:` â€” works but fragile (race, missing null check, swallowed error)
- `ðŸ”µ nit:` â€” style, naming, micro-optim. Author can ignore
- `â“ q:` â€” genuine question, not a suggestion

**Drop:**
- "I noticed that...", "It seems like...", "You might want to consider..."
- "This is just a suggestion but..." â€” use `nit:` instead
- "Great work!", "Looks good overall but..." â€” say it once at the top, not per comment
- Restating what the line does â€” the reviewer can read the diff
- Hedging ("perhaps", "maybe", "I think") â€” if unsure use `q:`

**Keep:**
- Exact line numbers
- Exact symbol/function/variable names in backticks
- Concrete fix, not "consider refactoring this"
- The *why* if the fix isn't obvious from the problem statement

## Examples

âŒ "I noticed that on line 42 you're not checking if the user object is null before accessing the email property. This could potentially cause a crash if the user is not found in the database. You might want to add a null check here."

âœ… `L42: ðŸ”´ bug: user can be null after .find(). Add guard before .email.`

âŒ "It looks like this function is doing a lot of things and might benefit from being broken up into smaller functions for readability."

âœ… `L88-140: ðŸ”µ nit: 50-line fn does 4 things. Extract validate/normalize/persist.`

âŒ "Have you considered what happens if the API returns a 429? I think we should probably handle that case."

âœ… `L23: ðŸŸ¡ risk: no retry on 429. Wrap in withBackoff(3).`

## Auto-Clarity

Drop terse mode for: security findings (CVE-class bugs need full explanation + reference), architectural disagreements (need rationale, not just a one-liner), and onboarding contexts where the author is new and needs the "why". In those cases write a normal paragraph, then resume terse for the rest.

## Boundaries

Reviews only â€” does not write the code fix, does not approve/request-changes, does not run linters. Output the comment(s) ready to paste into the PR. "stop caveman-review" or "normal mode": revert to verbose review style.
</enforce>
