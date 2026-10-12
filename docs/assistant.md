# The built-in assistant

> **Last reviewed:** 2026-10-12 · **Last updated:** 2026-10-12 · **Change:** major (in-app model download and setup, runtime recovery) · **Target:** Microsoft Excel (Microsoft 365)

Lattice's desktop builds include an assistant that runs **entirely on your machine**: a fine-tuned Gemma model in a
local sidecar process. Nothing you type and no workbook data leaves the device. Open it with **Home → Assistant** or the
`assistant.toggle` command. The first time, it offers to download a model (see [Installing a model](#installing-a-model));
after that, nothing goes over the network.

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

| Model | File | Download | Recommended when |
|---|---|---|---|
| Standard (E4B) | `lattice-e4b-Q4_K_M.gguf` (Gemma-4-E4B fine-tune, 4-bit) | 5.3 GB | ≥ 15 GB RAM (Apple Silicon uses Metal) |
| Compact (E2B) | `lattice-e2b-Q4_K_M.gguf` (Gemma-4-E2B fine-tune, 4-bit) | 3.4 GB | smaller machines, CPU-only 8 GB laptops |

### Installing a model

Models aren't part of the app download. The first time the Assistant pane opens, it shows both models with what each
is good and bad at, badges the one that suits this computer (by memory size and GPU) and offers **Download**. Both
can be installed; **Models** in the pane header switches between them or removes one.

- **Downloads are safe to interrupt.** The file grows as `<name>.part` and resumes where it stopped (after a cut
  connection, Cancel or quitting the app). It is used only after its size and SHA-256 match the values pinned in
  `crates/assistant/src/models.rs`; a damaged file is deleted, never loaded. Free disk space is checked first.
- **Where they go.** A per-user folder outside the app bundle: `~/Library/Application Support/Lattice/assistant/models`
  (macOS), `%LOCALAPPDATA%\Lattice\assistant\models` (Windows), `$XDG_DATA_HOME/lattice/assistant/models` (Linux/BSD).
- **Offline.** "Use a file I already have…" verifies a `.gguf` the user picks and copies it in.
- **Where from.** The download sources are a list in `models.rs` (today Google Drive "anyone with the link" files;
  Drive's large-file and quota pages are recognised, not saved). `LATTICE_ASSISTANT_MODEL_URL` adds a mirror tried
  first (base URL; the file name is appended).

### Runtime

`llama-server` (`llama-server.exe` on Windows) and its libraries ship with desktop releases; they are never committed
(large binaries). It is looked up in `LATTICE_ASSISTANT_DIR`, then `assistant/` next to the executable, then the macOS
bundle's `Helpers/assistant/` (or `Resources/assistant/`).

Starting it never takes the app down and recovers on its own where it can:

- a GPU (Metal) failure retries on the CPU; running out of memory retries with a shorter context (16k down to 4k;
  this saves little, since the memory is mostly the model's weights) and then on the CPU; a port taken by another
  program retries on a new port;
- the server's log is kept, so a failure says why (memory, GPU, damaged model) instead of a bare exit code;
- if the model process dies, the next request restarts it; if it dies mid-request before anything changed, the request
  is retried once;
- a server left behind by a crashed app is stopped at the next start (`llama-server.pid` next to the models folder).

Overrides: `LATTICE_ASSISTANT_TIER=e4b|e2b`, `LATTICE_ASSISTANT_GPU_LAYERS=N`, `LATTICE_ASSISTANT_MODELS_DIR=path`.

### Guards on the model's answers

- If every action in a request failed (for example, the model named a tool that doesn't exist) but the model still
  reports success, the pane adds "None of the assistant's actions worked, so the workbook wasn't changed."
- If the same step returns the same result three times in a row, the request stops instead of looping until the
  10-turn limit.

The web build has no assistant (`assistant.*` commands return an error there).

## Commands

| Command | Params | Does |
|---|---|---|
| `assistant.toggle` | `{open?: bool}` | Shows or hides the Assistant pane |
| `assistant.ask` | `{prompt}` | Opens the pane and asks; the reply appears in the pane |
| `assistant.models` | — | Installed, downloading and recommended models, download progress and the runtime state |
| `assistant.install` | `{tier: "e4b"\|"e2b"\|"both"}` | Downloads and verifies a model (installed ones are skipped) |
| `assistant.importModel` | `{tier, path}` | Verifies a model file the user already has and installs a copy |
| `assistant.cancelInstall` | `{tier?}` | Stops a download (all without `tier`); the next Download resumes it |
| `assistant.removeModel` | `{tier}` | Deletes a downloaded model and its partial download |
| `assistant.useModel` | `{tier}` | Switches to an installed model (remembered) and restarts the runtime |
| `assistant.manageModels` | `{open?: bool}` | Shows or hides the model setup screen in the pane |
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
| 2026-10-12 | major | In-app model setup: choose/download/verify/remove models, runtime fallbacks and restart, answer guards, `assistant.*` model commands |
