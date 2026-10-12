# The built-in assistant

> **Last reviewed:** 2026-10-12 · **Last updated:** 2026-10-12 · **Change:** major (created with the `lattice-assistant` crate and the Assistant pane) · **Target:** Microsoft Excel (Microsoft 365)

Lattice's desktop builds include an assistant that runs **entirely on your machine**: a fine-tuned Gemma model in a
local sidecar process. Nothing you type and no workbook data leaves the device. Open it with **Home → Assistant** or the
`assistant.toggle` command.

It does four jobs:

| Job | Example |
|---|---|
| Change the workbook | "Sort by revenue, highest first", "Add a Profit column", "Chart sales by region" |
| Write, explain or fix formulas | "Total revenue for North in H2", "Why does G7 show #DIV/0!?" |
| Answer questions about the data | "Which region grew fastest?", "Summarize this sheet" |
| Explain Lattice | "Where is Remove Duplicates?", "How do I script this with lattice-cli?" |

## How it works

- **Same tools as external agents.** The model calls 15 tools: `inspect_workbook`, `read_range`, `get_cell`, `set_cell`,
  `write_range`, `evaluate_formula`, `format_range`, `sort_range`, `filter`, `create_table`, `insert_chart`,
  `execute_command` (any of Lattice's commands), `list_commands`, `list_functions` and `lattice_docs`. All but the last
  run through `lattice_mcp::call_tool`, the code behind the [MCP server](mcp.md), against the live session.
- **Workbook snapshot.** Each request starts with a short summary of the workbook (sheets, used ranges, tables, charts)
  and the top-left of the active sheet as the user sees it (dates, currency and percentages as displayed; formula
  cells as `=FORMULA→value`). The model reads more with `read_range` when it needs to.
- **Docs lookup.** `lattice_docs` searches the documentation bundled into the build (this folder's CLI, MCP, control
  protocol and import pages, plus the README), so how-to answers follow the version you're running.
- **Thinking only when it pays.** Multi-step requests, long requests and math use the model's reasoning channel; quick
  edits, look-ups and how-to questions don't (`router::use_thinking`). On a laptop CPU reasoning costs seconds per step.
- **Undo.** Each request is one undo step (`DocState::squash_history_since`; agents can do the same with
  `history.squash {count, label}`).
- **Asks before risky actions.** `file.*`, `app.*` and `window.*` commands and deleting sheets wait for **Allow** in the
  pane; **Don't allow** tells the model, which then explains what it would have done.
- **Never crashes the app.** The model runs in a separate `llama-server` process; if it fails, the pane reports an error
  and the workbook is untouched. Model output is parsed defensively (malformed tool calls are skipped) and each request
  stops after 10 model turns.

## Models and hardware

| Tier | Model | Chosen when |
|---|---|---|
| E4B | `lattice-e4b-Q4_K_M.gguf` (Gemma-4-E4B fine-tune, 4-bit) | ≥ 15 GB RAM (Apple Silicon uses Metal) |
| E2B | `lattice-e2b-Q4_K_M.gguf` (Gemma-4-E2B fine-tune, 4-bit) | smaller machines, CPU-only 8 GB laptops |

Overrides: `LATTICE_ASSISTANT_TIER=e4b|e2b`, `LATTICE_ASSISTANT_GPU_LAYERS=N`.

The runtime and models are looked up in `LATTICE_ASSISTANT_DIR`, then an `assistant/` folder next to the executable,
then the macOS bundle's `Resources/assistant/`. The folder holds `llama-server` (`llama-server.exe` on Windows) and the
`.gguf` files. Release packaging puts them there; they are never committed to this repository (large binaries).

The web build has no assistant (`assistant.*` commands return an error there).

## Commands

| Command | Params | Does |
|---|---|---|
| `assistant.toggle` | `{open?: bool}` | Shows or hides the Assistant pane |
| `assistant.ask` | `{prompt}` | Opens the pane and asks; the reply appears in the pane |
| `history.squash` | `{count: 2.., label?}` | Merges the last `count` undo steps into one (any agent can use it) |

## For contributors

The assistant lives in `crates/assistant` (layer 6, depends on `engine` and `mcp`; `ui-egui` hosts it, desktop only).
The model was trained against a reference harness in the training repository; the prompt, the tool-observation text,
the tool-call parser, the docs index and the chat rendering must stay byte-identical to it. The golden tests in
`crates/assistant/tests/` enforce that, so a change to any of them needs the training side updated (and usually a new
model).

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-12 | major | Created: local assistant (harness, runtime sidecar, agent loop, pane), tiers, guardrails, one-step undo |
