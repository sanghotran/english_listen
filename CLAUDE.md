<!-- rtk-instructions v2 -->
# RTK (Rust Token Killer) - Token-Optimized Commands

## Golden Rule

**Always prefix commands with `rtk`**. If RTK has a dedicated filter, it uses it. If not, it passes through unchanged. This means RTK is always safe to use.

**Important**: Even in command chains with `&&`, use `rtk`:
```bash
# ❌ Wrong
git add . && git commit -m "msg" && git push

# ✅ Correct
rtk git add . && rtk git commit -m "msg" && rtk git push
```

## RTK Commands by Workflow

### Build & Compile (80-90% savings)
```bash
rtk cargo build         # Cargo build output
rtk cargo check         # Cargo check output
rtk cargo clippy        # Clippy warnings grouped by file (80%)
rtk tsc                 # TypeScript errors grouped by file/code (83%)
rtk lint                # ESLint/Biome violations grouped (84%)
rtk prettier --check    # Files needing format only (70%)
rtk next build          # Next.js build with route metrics (87%)
```

### Test (60-99% savings)
```bash
rtk cargo test          # Cargo test failures only (90%)
rtk go test             # Go test failures only (90%)
rtk jest                # Jest failures only (99.5%)
rtk vitest              # Vitest failures only (99.5%)
rtk playwright test     # Playwright failures only (94%)
rtk pytest              # Python test failures only (90%)
rtk rake test           # Ruby test failures only (90%)
rtk rspec               # RSpec test failures only (60%)
rtk test <cmd>          # Generic test wrapper - failures only
```

### Git (59-80% savings)
```bash
rtk git status          # Compact status
rtk git log             # Compact log (works with all git flags)
rtk git diff            # Compact diff (80%)
rtk git show            # Compact show (80%)
rtk git add             # Ultra-compact confirmations (59%)
rtk git commit          # Ultra-compact confirmations (59%)
rtk git push            # Ultra-compact confirmations
rtk git pull            # Ultra-compact confirmations
rtk git branch          # Compact branch list
rtk git fetch           # Compact fetch
rtk git stash           # Compact stash
rtk git worktree        # Compact worktree
```

Note: Git passthrough works for ALL subcommands, even those not explicitly listed.

### GitHub (26-87% savings)
```bash
rtk gh pr view <num>    # Compact PR view (87%)
rtk gh pr checks        # Compact PR checks (79%)
rtk gh run list         # Compact workflow runs (82%)
rtk gh issue list       # Compact issue list (80%)
rtk gh api              # Compact API responses (26%)
```

### JavaScript/TypeScript Tooling (70-90% savings)
```bash
rtk pnpm list           # Compact dependency tree (70%)
rtk pnpm outdated       # Compact outdated packages (80%)
rtk pnpm install        # Compact install output (90%)
rtk npm run <script>    # Compact npm script output
rtk npx <cmd>           # Compact npx command output
rtk prisma              # Prisma without ASCII art (88%)
```

### Files & Search (60-75% savings)
```bash
rtk ls <path>           # Tree format, compact (65%)
rtk read <file>         # Code reading with filtering (60%)
rtk grep <pattern>      # Search grouped by file (75%)
rtk find <pattern>      # Find grouped by directory (70%)
```

### Analysis & Debug (70-90% savings)
```bash
rtk err <cmd>           # Filter errors only from any command
rtk log <file>          # Deduplicated logs with counts
rtk json <file>         # JSON structure without values
rtk deps                # Dependency overview
rtk env                 # Environment variables compact
rtk summary <cmd>       # Smart summary of command output
rtk diff                # Ultra-compact diffs
```

### Infrastructure (85% savings)
```bash
rtk docker ps           # Compact container list
rtk docker images       # Compact image list
rtk docker logs <c>     # Deduplicated logs
rtk kubectl get         # Compact resource list
rtk kubectl logs        # Deduplicated pod logs
```

### Network (65-70% savings)
```bash
rtk curl <url>          # Compact HTTP responses (70%)
rtk wget <url>          # Compact download output (65%)
```

### Meta Commands
```bash
rtk gain                # View token savings statistics
rtk gain --history      # View command history with savings
rtk discover            # Analyze Claude Code sessions for missed RTK usage
rtk proxy <cmd>         # Run command without filtering (for debugging)
rtk init                # Add RTK instructions to CLAUDE.md
rtk init --global       # Add RTK to ~/.claude/CLAUDE.md
```

## Token Savings Overview

| Category | Commands | Typical Savings |
|----------|----------|-----------------|
| Tests | vitest, playwright, cargo test | 90-99% |
| Build | next, tsc, lint, prettier | 70-87% |
| Git | status, log, diff, add, commit | 59-80% |
| GitHub | gh pr, gh run, gh issue | 26-87% |
| Package Managers | pnpm, npm, npx | 70-90% |
| Files | ls, read, grep, find | 60-75% |
| Infrastructure | docker, kubectl | 85% |
| Network | curl, wget | 65-70% |

Overall average: **60-90% token reduction** on common development operations.
<!-- /rtk-instructions -->

<!-- gitnexus-instructions v1 -->
# GitNexus — Code Knowledge Graph (Required)

## Golden Rule

**Before exploring or changing code in this repo, use the `gitnexus` MCP tools instead of (or before) raw `grep`/manual file reading.** GitNexus keeps an indexed knowledge graph of the codebase (symbols, calls, imports, execution flows) and answers structural questions that text search cannot.

Skip GitNexus only for trivial, single-file, already-known lookups.

## Required workflow

1. **Discover the repo** (once per session, if needed):
   `mcp__gitnexus__list_repos` — confirm this project is indexed; pass `repo` explicitly on other calls if multiple repos are indexed.

2. **Before writing new code / understanding a feature**:
   `mcp__gitnexus__query` — natural-language search for existing execution flows related to the feature (e.g. "dictation diff scoring", "VOA RSS fetch"). Do this before re-implementing something that may already exist.

3. **Before editing a specific symbol** (function/class/component):
   `mcp__gitnexus__context` — get a 360° view (callers, callees, file location) of the symbol first.

4. **Before renaming, refactoring, or changing a shared symbol's signature/behavior**:
   `mcp__gitnexus__impact` (`direction: upstream`) — check blast radius and risk (LOW/MEDIUM/HIGH/CRITICAL) BEFORE making the change. Treat any `d=1` (WILL BREAK) result as mandatory to review/fix.

5. Use `mcp__gitnexus__cypher` / `mcp__gitnexus__group_query` only for advanced graph queries not covered by the above (rare).

## When GitNexus is stale

If GitNexus results look outdated (missing recently added files/symbols), run `mcp__gitnexus__detect_changes` (or ask the user to re-sync/re-index) rather than silently trusting stale results.

## Non-negotiable

- Do NOT skip `impact()` before modifying a function/component/Rust command that is called from more than one place (this repo's `src-tauri/src/commands/*`, `src/services/*`, and shared `src/store/*`/`src/hooks/*` are the highest-risk shared surfaces).
- Do NOT reach for `grep`/Explore-agent-only exploration as the first step when GitNexus is available — GitNexus first, grep to confirm/supplement.
<!-- /gitnexus-instructions -->
