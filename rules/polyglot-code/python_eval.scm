; Staged external Tree-sitter SAST query (Zero Binary Bloat)
; Rule: semgrep.python.dynamic-exec (CWE-95)
((call function: (identifier) @fn) (#match? @fn "^(eval|exec)$")) @hit
