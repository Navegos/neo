---
trigger: always_on
description: Enforce strict pure 7-bit ASCII (0x00-0x7F) on disk across all source, config, schema, query, and internal files while preserving intentional non-ASCII string literals via native ASCII escapes and cleaning mojibake in Markdown docs.
---

# Strict Pure 7-Bit ASCII Policy (0x00-0x7F Only)

## 1. Scope & Exemptions

### A. Enforced Paths (Strict 7-Bit ASCII `0x00`-`0x7F` on Disk)
- **All Systems, Native & Compiled Source Code:**
  - Rust: `*.rs`
  - C / C++ / CUDA / HIP / OpenCL / Objective-C: `*.c`, `*.cc`, `*.cp`, `*.cpp`, `*.cxx`, `*.c++`, `*.h`, `*.hh`, `*.hp`, `*.hpp`, `*.hxx`, `*.h++`, `*.inl`, `*.inc`, `*.ipp`, `*.ixx`, `*.tcc`, `*.tpp`, `*.txx`, `*.ccm`, `*.cppm`, `*.cxxm`, `*.c++m`, `*.i`, `*.ii`, `*.m`, `*.mm`, `*.cu`, `*.cuh`, `*.hip`, `*.cl`
  - C# / .NET / F# / VB: `*.cs`, `*.csx`, `*.fs`, `*.fsi`, `*.fsx`, `*.vb`, `*.vbx`
  - Go / Zig / Swift / Kotlin / Java / Scala / Groovy: `*.go`, `*.mod`, `*.sum`, `*.zig`, `*.swift`, `*.kt`, `*.kts`, `*.java`, `*.scala`, `*.sc`, `*.sbt`, `*.groovy`, `*.gradle`
- **All Scripting, Web, VM & Functional Source Code:**
  - Python & Cython: `*.py`, `*.pyi`, `*.pyw`, `*.pyz`, `*.pyt`, `*.pyx`, `*.pxd`, `*.pxi`
  - TypeScript / JavaScript / Web Frameworks: `*.ts`, `*.tsx`, `*.mts`, `*.cts`, `*.js`, `*.jsx`, `*.mjs`, `*.cjs`, `*.vue`, `*.svelte`, `*.astro`, `*.html`, `*.htm`, `*.css`, `*.scss`, `*.sass`, `*.less`
  - Lua / Luau / Ruby / PHP / Perl / R / Julia / Dart / Elixir / Erlang / Haskell / OCaml / Clojure: `*.lua`, `*.luau`, `*.rb`, `*.rake`, `*.gemspec`, `*.php`, `*.phtml`, `*.pl`, `*.pm`, `*.r`, `*.R`, `*.jl`, `*.dart`, `*.ex`, `*.exs`, `*.erl`, `*.hrl`, `*.hs`, `*.lhs`, `*.ml`, `*.mli`, `*.clj`, `*.cljs`, `*.cljc`, `*.edn`
  - Shell, PowerShell & Batch Scripts: `*.sh`, `*.bash`, `*.zsh`, `*.fish`, `*.ksh`, `*.dash`, `*.command`, `*.bats`, `*.ps1`, `*.psm1`, `*.psd1`, `*.ps1xml`, `*.bat`, `*.cmd`
- **All Configuration, Serialization, Schema & Query Files:**
  - TOML / INI / Properties / Dotenv: `*.toml`, `*.ini`, `*.cfg`, `*.conf`, `*.properties`, `*.env`, `*.env.*`, `.editorconfig`, `.gitattributes`, `.gitignore`, `.dockerignore`
  - JSON & JSON-Like: `*.json`, `*.json5`, `*.jsonc`, `*.jsonl`, `*.ndjson`, `*.webmanifest`, `*.webapp`, `*.avsc`, `*.geojson`, `*.topojson`, `*.har`
  - YAML & YAML-Based Configs: `*.yaml`, `*.yml`, `*.eyaml`, `*.eyml`, `*.raml`, `*.sls`, `*.condarc`, `*.gemrc`, `.clang-format`, `.clang-tidy`, `.prettierrc`, `.stylelintrc`
  - Rust Object Notation, KDL, HOCON, Cue, Dhall, Jsonnet & Nix: `*.ron`, `*.kdl`, `*.hocon`, `*.cue`, `*.dhall`, `*.jsonnet`, `*.libsonnet`, `*.nix`
  - XML & MSBuild / Project Files: `*.xml`, `*.xsd`, `*.xsl`, `*.xslt`, `*.wsdl`, `*.svg`, `*.plist`, `*.csproj`, `*.vbproj`, `*.fsproj`, `*.vcxproj`, `*.vcxproj.filters`, `*.proj`, `*.projitems`, `*.shproj`, `*.props`, `*.targets`, `*.ruleset`, `*.config`, `*.nuspec`, `*.resx`, `*.vsixmanifest`, `*.vsct`
  - SQL, Database & Schema Definitions: `*.sql`, `*.ddl`, `*.dml`, `*.pgsql`, `*.plsql`, `*.tsql`, `*.mysql`, `*.sqlite`, `*.cql`, `*.cypher`, `*.graphql`, `*.gql`, `*.prisma`, `*.proto`, `*.thrift`, `*.capnp`, `*.fbs`, `*.asn`, `*.asn1`
  - Infrastructure, Build, Policy & CI/CD Files: `Dockerfile`, `*.dockerfile`, `Containerfile`, `Makefile`, `*.mk`, `*.cmake`, `CMakeLists.txt`, `Justfile`, `*.just`, `*.tf`, `*.tfvars`, `*.hcl`, `*.bicep`, `*.rego`, `*.cedar`, `*.cedarschema`, `*.service`, `*.socket`, `*.timer`, `*.nginx`
- **All Internal Architecture, Research & Agentic Files:**
  - Architecture & Research Specs: `**/research/**`, `**/RESEARCHPLAN.md`, `**/RESEARCHPLAN.txt`
  - Agent Rules, Skills, Hooks, Prompts & Phase Plans: `.agents/**`, `.claude/**`, `.agents/brain/**`, `.cline/**`, `.codex/**`, `.copilot/**`, `.cursor/**`, `.gemini/**`, `.github/**`, `.grok/**`, `.toksi/**`, `agents/**`, `.clinerules`, `.cursorrules`, `.windsurfrules`, `AGENTS.md`, `CLAUDE.md`, `CODEX.md`, `GEMINI.md`

### B. Exempted Paths (Standard UTF-8 Allowed, Mojibake Forbidden)
- **Public Documentation & General Markdown:** Root and package `README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `LICENSE*`, and `docs/public/**` may use standard UTF-8 badges, symbols, or emojis, **except** corrupted Windows-1252 mojibake sequences (such as `\u00E2\u20AC\u201D`, `\u00E2\u2020\u2019`, `\u00E2\u201D\u0153`, `\u00C2\u00B7`), which MUST still be repaired across all Markdown files.
- **Vendored, Build & Binary Directories:** `.agents/vendor/**`, `vendor/**`, `third_party/**`, `node_modules/**`, `target/**`, `dist/**`, `build/**`, `.git/**`, and binary assets MUST NOT be modified unless explicitly instructed.

## 2. Core Mandate (Enforced Paths)
- **7-Bit ASCII Only on Disk:** Every enforced file created, edited, or saved MUST contain **only pure 7-bit ASCII characters (`0x00`-`0x7F`)** with `LF` (`\n`) line endings and zero UTF-8 BOM (`\uFEFF`).
- **Comments & Docstrings (`//`, `/* */`, `///`, `#`, `--`):** Code comments, docstrings, and internal Markdown docs MUST NEVER contain raw non-ASCII characters OR Unicode escape clutter; always normalize comments and docstrings to plain ASCII text using Section 4.
- **String Literals in Code, Configs & SQL/DB:** When source code, JSON/TOML configs, HTML/CSS templates, or SQL/database string values intentionally require non-ASCII symbols for UI/page presentation, Markdown output, i18n, or UTF-8/UTF-16 parser tests, **keep the file 100% pure 7-bit ASCII on disk** by encoding the characters with language-native ASCII escapes (Section 3) instead of stripping them or leaving raw UTF-8 bytes.
- **Markdown-Safe Normalization:** Always preserve ASCII backticks (`` ` ``, `0x60`) untouched so inline code spans and triple-backtick code fences never break. Map middle dots (`\u00B7`) and bullets (`\u2022`) to `-` (never `|` or `*`) so Markdown table columns and inline emphasis never break.
- **Why:** Prevents Windows-1252 / UTF-8 mojibake corruption across PowerShell, Rust/C++ builds, terminals, and cross-platform editors while preserving exact runtime UTF-8/UTF-16 string values.

## 3. Native ASCII Escapes for Runtime String Literals (Code, Config & DB)

Use these **pure 7-bit ASCII escape sequences** inside string literals (never inside source comments or docstrings) whenever runtime output requires non-ASCII symbols, emojis, or multi-byte characters:

| Language / Format | BMP Escape (`U+0000`..`U+FFFF`) | Non-BMP / Emoji Escape (`U+10000`..`U+10FFFF`) | Example (`U+1F600` Grinning Face) |
| :--- | :--- | :--- | :--- |
| **Rust** (`*.rs`) | `\u{XXXX}` | `\u{XXXXX}` | `"\u{1F600}"` |
| **C / C++ / CUDA / HIP / OpenCL / Obj-C** | `\uXXXX` | `\U000XXXXX` | `"\U0001F600"` |
| **C# / .NET / F# / VB** | `\uXXXX` | `\U000XXXXX` (or surrogate `\uD83D\uDE00`) | `"\U0001F600"` |
| **Go / Zig** | `\uXXXX` (Go) / `\u{XXXX}` (Zig) | `\U000XXXXX` (Go) / `\u{XXXXX}` (Zig) | `"\U0001F600"` / `"\u{1F600}"` |
| **Swift** (`*.swift`) | `\u{XXXX}` | `\u{XXXXX}` | `"\u{1F600}"` |
| **Kotlin / Java / Scala / Groovy** | `\uXXXX` | UTF-16 surrogate pair `\uD83D\uDE00` | `"\uD83D\uDE00"` |
| **Python & Cython** (`*.py`, `*.pyx`) | `\uXXXX` | `\U000XXXXX` | `"\U0001F600"` |
| **TypeScript / JavaScript / Web Frameworks** | `\uXXXX` or `\u{XXXX}` | `\u{XXXXX}` (ES6+) or `\uD83D\uDE00` | `"\u{1F600}"` |
| **HTML / SVG / XML Templates** | `&#xXXXX;` | `&#xXXXXX;` | `&#x1F600;` |
| **CSS / SCSS / Sass / Less** | `\XXXX ` | `\XXXXX ` | `content: "\1F600 ";` |
| **Lua / Luau** (`*.lua`, `*.luau`) | `\u{XXXX}` (Lua 5.3+/Luau) | `\u{XXXXX}` | `"\u{1F600}"` |
| **Ruby** (`*.rb`) | `\uXXXX` or `\u{XXXX}` | `\u{XXXXX}` | `"\u{1F600}"` |
| **PHP** (`*.php`, `*.phtml`) | `\u{XXXX}` (PHP 7+) | `\u{XXXXX}` | `"\u{1F600}"` |
| **Perl** (`*.pl`, `*.pm`) | `\x{XXXX}` | `\x{XXXXX}` | `"\x{1F600}"` |
| **R / Julia / Dart** | `\uXXXX` (R/Dart) / `\uXXXX` (Julia) | `\U000XXXXX` (R/Julia) / `\u{XXXXX}` (Dart) | `"\U0001F600"` / `"\u{1F600}"` |
| **Elixir / Erlang** | `\uXXXX` (Elixir) / `\x{XXXX}` (Erlang) | `\u{XXXXX}` (Elixir) / `\x{XXXXX}` (Erlang) | `"\u{1F600}"` / `"\x{1F600}"` |
| **Haskell / OCaml / Clojure** | `\xXXXX` (HS) / `\u{XXXX}` (OCaml) / `\uXXXX` (CLJ) | `\xXXXXX` (HS) / `\u{XXXXX}` (OCaml) / `\uD83D\uDE00` (CLJ) | `"\x1F600"` / `"\u{1F600}"` |
| **TOML** (`*.toml`) | `\uXXXX` (no braces) | `\U000XXXXX` (no braces) | `"\U0001F600"` |
| **JSON & JSON-Like** (`*.json`, `*.jsonc`) | `\uXXXX` | UTF-16 surrogate pair `\uD83D\uDE00` | `"\uD83D\uDE00"` |
| **SQL & Database Strings** (`*.sql`, Postgres, MySQL, SQLite) | `U&'\XXXX'` (PG) / `CHAR(0x... USING utf8mb4)` (MySQL) / `char(0xXXXX)` (SQLite) | `U&'\+0XXXXX'` (PG) / `CHAR(0xF09F9880 USING utf8mb4)` (MySQL) / `char(0x1F600)` (SQLite) | `U&'\+01F600'` |

### Rules for Preserving vs. Normalizing String Literals
1. **Preserve via Escapes (Do NOT Strip):** User-facing UI text, HTML/web page presentation templates, public Markdown generators, i18n localization strings, database seed strings storing user-facing symbols, and unit tests checking exact UTF-8 byte offsets or UTF-16 code units MUST use the native ASCII escapes above.
2. **Normalize to Plain ASCII (Section 4):** Source code comments (`//`, `/* */`, `///`, `#`, `--`), docstrings, internal developer logs, CLI diagnostic output, and internal Markdown docs MUST be normalized to plain ASCII using Section 4 (never leave raw UTF-8 or `\uXXXX` escapes in comments).

## 4. Mandatory ASCII Substitution Table (Comments, Docs & Internal Text)

### A. Directory Trees & Box-Drawing Tables

| Forbidden Unicode (`\uXXXX`) | UTF-8 / Mojibake Bytes | Required ASCII | Usage |
| :--- | :--- | :--- | :--- |
| `\u251C\u2500\u2500`, `\u2523\u2501\u2501`, `\u251D\u2501\u2501`, `\u2520\u2500\u2500` | `0xE2 0x94 0x9C ...` | `|--` | Directory tree branch |
| `\u2514\u2500\u2500`, `\u2517\u2501\u2501`, `\u2515\u2501\u2501`, `\u2516\u2500\u2500`, `\u2570\u2500\u2500` | `0xE2 0x94 0x94 ...` | `` `-- `` | Last directory tree branch |
| `\u2502`, `\u2503`, `\u2551`, `\u00A6`, `\uFF5C` | `0xE2 0x94 0x82`, `0xC2 0xA6` | `|` | Vertical pipe / divider |
| `\u2500`, `\u2501`, `\u2504`, `\u2505`, `\u2508`, `\u2509`, `\u23AF` | `0xE2 0x94 0x80` | `-` | Horizontal line |
| `\u2550` | `0xE2 0x95 0x90` | `=` | Double horizontal line |
| `\u250C`, `\u2510`, `\u2514`, `\u2518`, `\u251C`, `\u2524`, `\u252C`, `\u2534`, `\u253C`, `\u2554`, `\u2557`, `\u255A`, `\u255D`, `\u2560`, `\u2563`, `\u2566`, `\u2569`, `\u256C`, `\u256D`-`\u2570` | `0xE2 0x94 0x8C ...` | `+` | All box corners & junctions |

### B. Dashes, Quotes, Bullets & Punctuation (Markdown-Safe)

| Forbidden Unicode (`\uXXXX`) | UTF-8 / Mojibake Bytes | Required ASCII | Usage |
| :--- | :--- | :--- | :--- |
| `\u2014`, `\u2015`, `\u2E3A`, `\u2E3B` | `0xE2 0x80 0x94` (and `0x22` variant) | `---` | Em dash / sentence break |
| `\u2013`, `\u2012`, `\u2010`, `\u2011`, `\u2212` | `0xE2 0x80 0x93` | `--` | En dash / numeric range (`30--521`) |
| `\u2019`, `\u2018`, `\u201A`, `\u201B`, `\u2032`, `\u00B4` | `0xE2 0x80 0x99` (and `0x27` variant) | `'` | Single quote / apostrophe (keep `` ` `` untouched) |
| `\u201C`, `\u201D`, `\u201E`, `\u201F`, `\u2033`, `\u300C`, `\u300E`, `\u300D`, `\u300F` | `0xE2 0x80 0x9C` / `0x9D` | `"` | Double quote / CJK corner brackets |
| `\u00AB`, `\u300A` | `0xC2 0xAB` | `<<` | Left guillemet / book title mark |
| `\u00BB`, `\u300B` | `0xC2 0xBB` | `>>` | Right guillemet / book title mark |
| `\u2039` | `0xE2 0x80 0xB9` | `<` | Single left-pointing angle quote |
| `\u203A` | `0xE2 0x80 0xBA` | `>` | Single right-pointing angle quote |
| `\u2E28`, `\u2E29` | `0xE2 0xB8 0xA8` / `0xA9` | `((`, `))` | Double parentheses |
| `\u00B7`, `\u2219`, `\u22C5` | `0xC2 0xB7` | `-` | Inline metadata separator (safe in Markdown tables) |
| `\u2022`, `\u25E6`, `\u25AA`, `\u25AB`, `\u25CF`, `\u25CB`, `\u25C6`, `\u25C7`, `\u25B8`, `\u25B9`, `\u25BA`, `\u25B6`, `\u2023`, `\u2043` | `0xE2 0x80 0xA2` | `-` | List bullets (avoids accidental `*` emphasis) |
| `\u2026`, `\u22EF`, `\u22EE`, `\u22F0`, `\u22F1` | `0xE2 0x80 0xA6` | `...` | Ellipsis |
| `\u3001`, `\uFF0C` | `0xE3 0x80 0x81`, `0xEF 0xBC 0x8C` | `, ` | CJK enumeration / fullwidth comma |
| `\u3002` | `0xE3 0x80 0x82` | `.` | CJK full stop |
| `\uFF1A` | `0xEF 0xBC 0x9A` | `: ` | Fullwidth colon |
| `\uFF1B` | `0xEF 0xBC 0x9B` | `; ` | Fullwidth semicolon |
| `\uFF08`, `\u3010` | `0xEF 0xBC 0x88`, `0xE3 0x80 0x90` | `(` | Fullwidth / lenticular left bracket |
| `\uFF09`, `\u3011` | `0xEF 0xBC 0x89`, `0xE3 0x80 0x91` | `)` | Fullwidth / lenticular right bracket |
| `\uFF01`, `\uFF1F` | `0xEF 0xBC 0x81` / `0x9F` | `!`, `?` | Fullwidth exclamation & question mark |
| `\u00A0`, `\u2000`-`\u200A`, `\u202F`, `\u205F`, `\u3000` | `0xC2 0xA0` | ` ` (`0x20`) | Standard ASCII space |
| `\u1680` | `0xE1 0x9A 0x80` | `[OGHAM_SPACE]` | Ogham space mark (explicit tag in docs/comments) |
| `\u200B`, `\u200C`, `\u200D`, `\u2060`, `\uFE00`-`\uFE0F`, `\uFEFF` | `0xE2 0x80 0x8B`, `0xEF 0xBB 0xBF` | *(empty)* | Strip zero-width, variation selectors & BOM bytes |

### C. Arrows & Flow Indicators

| Forbidden Unicode (`\uXXXX`) | UTF-8 / Mojibake Bytes | Required ASCII | Usage |
| :--- | :--- | :--- | :--- |
| `\u2192`, `\u2197`, `\u2198`, `\u21B3`, `\u2794`, `\u279C`, `\u27A1`, `\u27F6`, `\u21A6` | `0xE2 0x86 0x92`, `0xE2 0x86 0xB3` | `->` | Right / diagonal-right / bent-right arrow |
| `\u2190`, `\u2196`, `\u2199`, `\u21B2`, `\u25C0`, `\u25C4`, `\u25C2`, `\u25C3`, `\u27F5`, `\u21A4` | `0xE2 0x86 0x90`, `0xE2 0x86 0xB2` | `<-` | Left / diagonal-left / bent-left arrow |
| `\u2191`, `\u21D1`, `\u25B2`, `\u25B4`, `\u2B61` | `0xE2 0x86 0x91` | `^` | Up arrow / triangle |
| `\u2193`, `\u21D3`, `\u25BC`, `\u25BE`, `\u25BD`, `\u2B63` | `0xE2 0x86 0x93` | `v` | Down arrow / triangle |
| `\u2194`, `\u27F7`, `\u21C4`, `\u21C6` | `0xE2 0x86 0x94`, `0xE2 0x87 0x84` | `<->` | Bidirectional / paired horizontal arrows |
| `\u2195`, `\u21D5` | `0xE2 0x86 0x95` | `|` | Bidirectional vertical arrow |
| `\u21D2`, `\u27F9`, `\u21E8` | `0xE2 0x87 0x92` | `=>` | Double right arrow / implication |
| `\u21D0`, `\u27F8`, `\u21E6` | `0xE2 0x87 0x90` | `<=` | Double left arrow |
| `\u21D4`, `\u27FA` | `0xE2 0x87 0x94` | `<=>` | Double bidirectional arrow |
| `\u276F`, `\u276E` | `0xE2 0x9D 0xAF` / `0xAE` | `>` | Heavy angle prompt / chevron |

### D. Math, Logic, Greek Letters, Superscripts & Fractions

| Forbidden Unicode (`\uXXXX`) | UTF-8 / Mojibake Bytes | Required ASCII | Usage |
| :--- | :--- | :--- | :--- |
| `\u2264`, `\u2A7D` | `0xE2 0x89 0xA4` | `<=` | Less than or equal |
| `\u2265`, `\u2A7E` | `0xE2 0x89 0xA5` | `>=` | Greater than or equal |
| `\u226A`, `\u226B` | `0xE2 0x89 0xAA` / `0xAB` | `<<`, `>>` | Much less-than / much greater-than |
| `\u2260` | `0xE2 0x89 0xA0` | `!=` | Not equal |
| `\u2248`, `\u2243`, `\u2245`, `\u221D` | `0xE2 0x89 0x88`, `0xE2 0x88 0x9D` | `~=` / `~` | Approximately equal / proportional to |
| `\u2261` | `0xE2 0x89 0xA1` | `===` | Identical / equivalent |
| `\u00B1` | `0xC2 0xB1` | `+/-` | Plus-minus |
| `\u00D7`, `\u2715`, `\u2716` | `0xC3 0x97` | `x` | Multiplication / dimension |
| `\u00F7` | `0xC3 0xB7` | `/` | Division |
| `\u222A`, `\u22C3` | `0xE2 0x88 0xAA` | `U` | Set union |
| `\u2229`, `\u22C2` | `0xE2 0x88 0xA9` | `&` | Set intersection |
| `\u2208` | `0xE2 0x88 0x88` | `in` | Element of |
| `\u2209` | `0xE2 0x88 0x89` | `not in` | Not element of |
| `\u2282`, `\u2286` | `0xE2 0x8A 0x82` | `subset` | Subset |
| `\u2283`, `\u2287` | `0xE2 0x8A 0x83` | `superset` | Superset |
| `\u2205` | `0xE2 0x88 0x85` | `{}` | Empty set |
| `\u2200` | `0xE2 0x88 0x80` | `forall` | Universal quantifier |
| `\u2203` | `0xE2 0x88 0x83` | `exists` | Existential quantifier |
| `\u00AC` | `0xC2 0xAC` | `!` | Logical NOT |
| `\u2227` | `0xE2 0x88 0xA7` | `&&` | Logical AND |
| `\u2228` | `0xE2 0x88 0xA8` | `||` | Logical OR |
| `\u2295` | `0xE2 0x8A 0x95` | `XOR` | Exclusive OR |
| `\u2218` | `0xE2 0x88 0x98` | `o` | Ring / composition operator |
| `\u221E` | `0xE2 0x88 0x9E` | `inf` | Infinity |
| `\u2211`, `\u220F`, `\u221A`, `\u2206`, `\u25B3`, `\u2207`, `\u2202` | `0xE2 0x88 0x91 ...` | `SUM`, `PROD`, `sqrt`, `DELTA`, `NABLA`, `d` | Calculus & aggregation |
| `\u03B1`, `\u03B4`, `\u03BB`, `\u03C3` | `0xCE 0xB1`, `0xCE 0xB4`, `0xCE 0xBB`, `0xCF 0x83` | `alpha`, `delta`, `lambda`, `stddev` | Greek variables & statistical notation |
| `\u2070`, `\u00B9`, `\u00B2`, `\u00B3`, `\u207F`, `\u2076\u2074` | `0xC2 0xB2 ...` | `^0`, `^1`, `^2`, `^3`, `^n`, `^64` | Superscript exponents |
| `\u00BD`, `\u00BC`, `\u00BE` | `0xC2 0xBD`, `0xC2 0xBC`, `0xC2 0xBE` | `1/2`, `1/4`, `3/4` | Vulgar fractions |
| `\u00B0`, `\u00B5`, `\u03BC`, `\u00A9`, `\u00AE`, `\u2122`, `\u00A7`, `\u00B6` | `0xC2 0xB0 ...` | ` deg`, `u`, `(c)`, `(R)`, `(TM)`, `Sec.`, `Para.` | Units & legal markers |

### E. Latin Diacritics, Ligatures & Keyboard Modifiers

| Forbidden Unicode (`\uXXXX`) | UTF-8 / Mojibake Bytes | Required ASCII | Usage |
| :--- | :--- | :--- | :--- |
| `\u00E9`, `\u00E8`, `\u00EA`, `\u00EB` (`\u00C9`-`\u00CB`) | `0xC3 0xA9 ...` | `e` (`E`) | Accented E |
| `\u00E0`, `\u00E2`, `\u00E4`, `\u00E1`, `\u00E3` | `0xC3 0xA0 ...` | `a` | Accented A |
| `\u00E7` (`\u00C7`) | `0xC3 0xA7` (`0x87`) | `c` | Cedilla C |
| `\u00F9`, `\u00FB`, `\u00FC`, `\u00FA` | `0xC3 0xB9 ...` | `u` | Accented U |
| `\u00F4`, `\u00F6`, `\u00F3`, `\u00F5` | `0xC3 0xB4 ...` | `o` | Accented O |
| `\u00EE`, `\u00EF`, `\u00ED` | `0xC3 0xAE ...` | `i` | Accented I |
| `\u00F1` | `0xC3 0xB1` | `n` | Tilde N |
| `\u0153` (`\u0152`) | `0xC5 0x93` (`0x92`) | `oe` (`OE`) | OE ligature |
| `\u2318`, `\u21E7` | `0xE2 0x8C 0x98`, `0xE2 0x87 0xA7` | `Cmd+`, `Shift+` | macOS shortcut modifier keys |

### F. Checkboxes, Blocks, Spinners, Circled Digits & Status Emojis

| Forbidden Unicode (`\uXXXX`) | UTF-8 / Mojibake Bytes | Required ASCII | Usage |
| :--- | :--- | :--- | :--- |
| `\u2713`, `\u2714`, `\u2611` | `0xE2 0x9C 0x93` / `0x94` | `[x]` | Checked / passing |
| `\u2717`, `\u2718`, `\u2612`, `\u2610`, `\u2B1C`, `\u2B1B` | `0xE2 0x9C 0x97`, `0xE2 0x98 0x90`, `0xE2 0xAC 0x9C` | `[ ]` | Unchecked / empty ballot box |
| `\u2705` | `0xE2 0x9C 0x85` | `[DONE]` | Completed / verified status |
| `\u274C` | `0xE2 0x9D 0x8C` | `[FAIL]` | Error / failed status |
| `\u26A0` | `0xE2 0x9A 0xA0` | `[WARN]` | Warning status |
| `\u2139`, `\u1F535` | `0xE2 0x84 0xB9`, `0xF0 0x9F 0x94 0xB5` | `[INFO]` / `[BLUE]` | Informational status / blue badge |
| `\u26D4`, `\u1F6AB` | `0xE2 0x9B 0x94`, `0xF0 0x9F 0x9A 0xAB` | `[BLOCKED]` | Blocked / prohibited |
| `\u2753`, `\u2754` | `0xE2 0x9D 0x93` / `0x94` | `[?]` | Unknown / question marker |
| `\u2605`, `\u2606`, `\u2B50`, `\u1F31F` | `0xE2 0x98 0x85` | `[*]` | Star / highlight marker |
| `\u2460`-`\u2469` | `0xE2 0x91 0xA0`-`0xA9` | `(1)`-`(10)` | Circled step numbers |
| `[0-9]\uFE0F?\u20E3` | `0x31 0xEF 0xB8 0x8F 0xE2 0x83 0xA3` | `[0]`-`[9]` | Keycap digits |
| `\u2588`, `\u2593`, `\u2592` | `0xE2 0x96 0x88` | `#` | Filled progress bar block |
| `\u2591` | `0xE2 0x96 0x91` | `.` | Empty progress bar shade |
| `\u2584`, `\u2580` | `0xE2 0x96 0x84` / `0x80` | `_`, `-` | Lower / upper half block |
| `\u25D0`-`\u25D3` | `0xE2 0x97 0x90`-`0x93` | `[HALF]` | Half-filled circle indicator |
| `\u25C9`, `\u25EF` | `0xE2 0x97 0x89`, `0xE2 0x97 0xAF` | `o` | Graph node / large circle |
| `\u2394` | `0xE2 0x8E 0x94` | `>` | CLI step bullet (`wrangler`) |
| `\u2800`-`\u28FF` | `0xE2 0xA0 0x80` ... | `*` | Braille spinner frames (use Section 3 escapes in code) |
| `\u1F534`, `\u1F7E1`, `\u1F7E2`, `\u1F7E0`, `\u26AA`, `\u26AB` | `0xF0 0x9F 0x94 0xB4 ...` | `[HIGH]`, `[MED]`, `[LOW]`, `[ORANGE]`, `[WHITE]` | Priority & status circle badges |
| `\u1F4A1`, `\u1F525`, `\u1F9E0`, `\u1F3AF`, `\u1F6A8` | `0xF0 0x9F 0x92 0xA1 ...` | `[TIP]`, `[HOT]`, `[IDENTITY]`, `[MISSION]`, `[CRITICAL]` | Callout & alert badges |
| `\u1F6E0`, `\u1F4CB`, `\u1F916`, `\u1F4E6`, `\u23ED` | `0xF0 0x9F 0x9B 0xA0 ...` | `[TOOLS]`, `[CHECKLIST]`, `[BOT]`, `[PKG]`, `[SKIP]` | Workflow & tooling badges |
| `\u1F50D`, `\u1F50E`, `\u1F680`, `\u26A1`, `\u1F527` | `0xF0 0x9F 0x94 0x8D ...` | `[INPUT]`, `[NEW]`, `[PERF]`, `[INTERNAL]` | Inspection & performance badges |
| `\u1F310`, `\u1F4DA`, `\u1F4D6`, `\u1F389`, `\u1F33F`, `\u1F4DD` | `0xF0 0x9F 0x8C 0x90 ...` | `[WEB]`, `[DOCS]`, `[CELEBRATE]`, `[FERN]`, `[NOTE]` | Docs, web & note badges |
| `\u1F4CA`, `\u1F980`, `\u1F517`, `\u1F9EA`, `\u1F9F9` | `0xF0 0x9F 0x93 0x8A ...` | `[METRICS]`, `[RUST]`, `[LINK]`, `[TEST]`, `[CLEAN]` | Analytics, Rust & test badges |
| `\u1F5D1`, `\u1F4C2`, `\u1F4C1`, `\u1F4BE`, `\u23F3`, `\u231B` | `0xF0 0x9F 0x97 0x91 ...` | `[DEL]`, `[DIR]`, `[DISK]`, `[WAIT]` | Filesystem & wait badges |
| `\u1F30D`-`\u1F30F`, `\u23F1`, `\u1F5A5`, `\u1F4BB`, `\u1F4B0`, `\u1F4B5`, `\u1F50C` | `0xF0 0x9F 0x8C 0x8D ...` | `[GLOBAL]`, `[TIME]`, `[OS]`, `[COST]`, `[PLUG]` | Architecture & infrastructure badges |
| `\u26C5`, `\u2699`, `\u1F522`, `\u1F528`, `\u1F3F7`, `\u1F6A2`, `\u1F4E1` | `0xE2 0x9B 0x85 ...` | `[WRANGLER]`, `[CONFIG]`, `[VER]`, `[BUILD]`, `[TAG]`, `[PUSH]`, `[FETCH]` | CLI, release & network badges |
| `\u2796`, `\u1F504`, `\u1F52C`, `\u1F3D7`, `\u270B`, `\u1F6A6`, `\u1F6A5` | `0xE2 0x9E 0x96 ...` | `[-]`, `[FLOW]`, `[RIG]`, `[DESIGN]`, `[STOP]`, `[GATE]` | Pipeline & gate badges |
| `\u1F3C3\u200D?\u2642`, `\u1F41E`, `\u1F41B`, `\u1F419`, `\u2728`, `\u1F5C4` | `0xF0 0x9F 0x8F 0x83 ...` | `[RUN]`, `[DEBUG]`, `[GITHUB]`, `[FEAT]`, `[DB]` | Execution, debug & release badges |
| `\u1F512`, `\u1F513`, `\u1F4DC`, `\u1F500`, `\u1F5E3`, `\u1F195`, `\u1F684` | `0xF0 0x9F 0x94 0x92 ...` | `[LOCK]`, `[LOG]`, `[BRANCH]`, `[SPEAK]`, `[NEW]`, `[INFRA]` | Security, git & infra badges |
| `\u1F44D`, `\u1F44E`, `\u2764`, `\u2665`, `\u1F370` | `0xF0 0x9F 0x91 0x8D ...` | `[+1]`, `<3`, `[CAKE]` | Reactions & celebratory tags |
| `\u1F600`-`\u1F642`, `\u1F622`-`\u1F641`, `\u1F621`-`\u1F92C`, `\u1F60E`, `\u1F914` | `0xF0 0x9F 0x98 0x80 ...` | `[HAPPY]`, `[SAD]`, `[ANGRY]`, `[COOL]`, `[THINK]` | Sentiment & mood tags |

## 5. Agent Verification Checklist
1. Before finalizing any file write or patch on an enforced path, verify that **zero bytes above `0x7F`** exist in the output on disk.
2. When migrating legacy `.txt` or external specifications (such as `RESEARCHPLAN.txt` -> `RESEARCHPLAN.md`), normalize all Unicode and Windows-1252 sequences (including straightened-quote variants) to the ASCII equivalents in Section 4 during conversion.
3. For string literals in code, TOML/JSON configs, HTML/CSS templates, or SQL/DB seed queries that intentionally render non-ASCII symbols for UI/page presentation, Markdown output, i18n, or UTF-8/UTF-16 parser tests, encode them using the language-native pure-ASCII escapes in Section 3 (never leave raw UTF-8 bytes on disk, and never put escape sequences inside comments or docstrings).
4. Preserve inline Markdown code backticks (`` ` ``, `0x60`) and triple-backtick code fences untouched.
5. On exempted Markdown paths (`README.md`, `CHANGELOG.md`, `docs/public/**`), allow clean UTF-8 but repair any broken Windows-1252 mojibake sequences immediately.
