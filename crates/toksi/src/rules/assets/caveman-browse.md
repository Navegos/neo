# caveman-browse

Local browser-interaction MCP specification using `Accessibility.getFullAXTree` compressed through the forced `a11y` tree compressor.

## Tools

- `browser_snapshot`: captures Chrome accessibility tree; optional `query` isolates best-matching nodes and ancestors; compact `[uid] role "name"` output with exact `tokens_after` and `view_tokens`.
- `browser_act`: interacts with resolved `[uid]` target; marks `settled: false` until follow-up focused snapshot proves DOM state.
- `browser_eval`: bounded evaluation against page context.
- `browser_recover`: returns byte-exact original accessibility payload from CCR handle.

## Safety

Navigation permits `http(s):`, `about:blank`, and bounded `data:text/html`; rejects local `file://` and privileged browser schemes.
