# AIMv2

[中文版](README-zh.md)

`aimv2` is a command-line AI mathematics assistant that runs in a local working directory. It can:

- explore mathematical problems and proof strategies;
- save intermediate statements, proofs, and their dependencies in a theorem graph;
- review proofs in multiple passes and record any issues found;
- save progress on long-running tasks so you can resume them later;
- read local materials, run code, and write result files with your permission.

> **Terminology:** In the theorem graph, a “theorem” is primarily a **statement**—a proposition or intermediate result that AIM formulates or derives while solving a problem. It is not necessarily an established theorem retrieved from the literature.

## Quick Start: Up and Running in 5 Minutes

### 1. Install Rust

AIMv2 requires Rust and Cargo. First, verify that both are available:

```bash
rustc --version
cargo --version
```

If either command is unavailable, install Rust from the [official Rust installation page](https://www.rust-lang.org/tools/install). macOS users with [Homebrew](https://brew.sh/) can also run:

```bash
brew install rust
```

### 2. Install AIMv2

From the root of this repository, run:

```bash
cargo install --path .
```

Verify the installation:

```bash
aimv2 --help
```

You can now run `aimv2` from any mathematics project directory. After updating the local source, reinstall and overwrite the existing binary with:

```bash
cargo install --path . --force
```

### 3. Create a Workspace and Configure the API

AIMv2 treats the directory from which you launch it as the workspace. Using a separate directory for each problem or project is recommended:

```bash
mkdir my-math-project
cd my-math-project
```

Create a `.env` file in that directory:

```dotenv
AIM_API_KEY=replace-with-your-api-key
AIM_BASE_URL=https://replace-with-your-provider-url/v1
```

For example, if your provider's endpoint is `https://endpoint.com/v1`, use:

```dotenv
AIM_API_KEY=replace-with-your-api-key
AIM_BASE_URL=https://endpoint.com/v1
```

If you use the official OpenAI API directly, you only need the official API key; no endpoint configuration is required:

```dotenv
OPENAI_API_KEY=replace-with-your-api-key
```

AIMv2 will then use the default endpoint, `https://api.openai.com/v1`. You can create or manage keys on the [OpenAI API Keys](https://platform.openai.com/settings/organization/api-keys) page.

Never commit a real API key to Git. Make sure your project's `.gitignore` includes `.env`.

### 4. Run AIMv2 for the First Time

The model name must exactly match one provided by your API provider. The following example uses `gpt-5.6-sol`:

```bash
# 1. Choose an appropriate directory and filename for the log.
# 2. Avoid placing it in the workspace, where AIMv2 might read it automatically
#    and introduce unnecessary confusion.
# 3. If --log-path is omitted, the log is stored in the system's default location.

aimv2 \
  --model gpt-5.6-sol \
  --reasoning-effort high \
  --enable-shell \
  --log-path YOUR_LOG_FILE.json
```

> **Security note:** `--enable-shell` allows AIM to execute commands and modify files with your current user permissions. It does not provide a complete security sandbox. For your first run, keep the default per-command confirmation mode and do not add `--auto`. If the task does not require reading files, running code, or writing results, omit `--enable-shell`.

Check the following fields in the startup information:

- `workspace`: Is this the project directory you just created?
- `model`: Does your provider support this model?
- `reasoning effort`: Is this the reasoning level you want?
- `shell tool`: Is its status what you expect?
- `history`: Where is the session log actually being saved?

Then enter your problem in natural language. For example:

```text
Study the following problem. First state the assumptions and goal clearly, then
explore possible proof strategies. Record reliable intermediate statements and
their dependencies in the theorem graph:

Let f:[0,1]→R be continuous and suppose ∫₀¹f(x)dx=0. Prove that there exists
ξ∈(0,1) such that ...
```

Enter `/help` to see the commands available within a session, or `/exit` to end the session.

For a deeper, more granular review by the progressive reviewer, increase its maximum number of iterations from within the session. For example:

```text
/iterations 5
```

More iterations generally mean more API calls, longer wait times, and higher costs, but they do not guarantee that the proof is correct.

## API and Model Configuration

### Environment Variables

AIMv2 reads configuration in the following order of precedence:

| Purpose | Primary variable | Fallback variable | If unset |
| --- | --- | --- | --- |
| API key | `AIM_API_KEY` | `OPENAI_API_KEY` | AIMv2 cannot start |
| API endpoint | `AIM_BASE_URL` | `OPENAI_BASE_URL` | `https://api.openai.com/v1` |

We recommend placing these settings in a `.env` file at the workspace root. AIMv2 automatically reads **only the `.env` file in the current workspace**; a `.env` file in the source repository used for installation will not apply automatically to other workspaces.

You can also set the variables temporarily for the current terminal session:

```bash
export AIM_API_KEY='replace-with-your-api-key'
export AIM_BASE_URL='https://replace-with-your-provider-url/v1'
```

### Choosing a Model

AIMv2 currently defaults to `gpt-5.6-sol`, but a third-party endpoint may not offer that model. The most reliable approach is to explicitly specify a model supported by your provider whenever you start a new session:

```bash
aimv2 --model gpt-5.6-sol
```

`gpt-5.6-sol` is only an example. Consult your provider's model list and use the exact model identifier.

You can also adjust the reasoning effort:

```bash
aimv2 --model gpt-5.6-sol --reasoning-effort high
```

Valid values are `minimal`, `low`, `medium`, and `high`. The default is `medium`.

## Recommended Project Structure

```text
my-math-project/
├── .env                    # API configuration; do not commit to Git
├── .gitignore
├── problem.md              # Problem statement, definitions, and conventions
└── notes.tex               # Optional: existing proof or draft
```

Always `cd` into the correct project directory before launching AIMv2. The workspace determines:

- which `.env` file is loaded automatically;
- the directory in which AIM is instructed to read, write, and run commands when `--enable-shell` is active;
- where relative log paths are saved;
- which historical sessions are searched by `resume --last` and `view --last`.

## Common Use Cases

### Use Case 1: Explore a Well-Defined Problem

```bash
# 1. Choose an appropriate directory and filename for the log.
# 2. Avoid placing it in the workspace, where AIMv2 might read it automatically
#    and introduce unnecessary confusion.
# 3. If --log-path is omitted, the log is stored in the system's default location.

cd my-math-project
aimv2 --model gpt-5.6-sol --reasoning-effort high --log-path YOUR_LOG_FILE.json
```

Example prompt:

```text
Help me investigate the problems I am working on. First check whether each
statement is likely to hold, then develop a proof or disproof step by step.
```

If this task does not require reading or writing local files, you can leave the shell disabled.

### Use Case 2: Turn a Vague Idea into a Precise Problem

```bash
# 1. Choose an appropriate directory and filename for the log.
# 2. Avoid placing it in the workspace, where AIMv2 might read it automatically
#    and introduce unnecessary confusion.
# 3. If --log-path is omitted, the log is stored in the system's default location.

aimv2 --model gpt-5.6-sol --reasoning-effort high --enable-shell --log-path YOUR_LOG_FILE.json
```

Example prompt:

```text
I want to study whether neural networks can discover new invariants in
combinatorics, but the question is not yet precise enough. Help me clarify the
objects of study, admissible assumptions, verifiable goals, and possible
counterexamples. Propose three candidate problems, ordered from weakest to
strongest. Do not claim that any of them has already been proved.
```

This repository includes `.aim/skills/problem-clarifier/SKILL.md`, which helps turn vague ideas into well-defined problems. Installing the binary with `cargo install` does not automatically copy repository skills into other workspaces. To use this skill in your own project, run:

```bash
mkdir -p .aim/skills
cp -R /path/to/AIMv2/.aim/skills/problem-clarifier .aim/skills/
```

Replace `/path/to/AIMv2` with the actual path to this repository. Skills are discovered only when `--enable-shell` is active.

### Use Case 3: Work with Existing LaTeX, Markdown, or Code

```bash
# 1. Choose an appropriate directory and filename for the log.
# 2. Avoid placing it in the workspace, where AIMv2 might read it automatically
#    and introduce unnecessary confusion.
# 3. If --log-path is omitted, the log is stored in the system's default location.

cd my-math-project
aimv2 \
  --model gpt-5.6-sol \
  --reasoning-effort high \
  --enable-shell \
  --log-path YOUR_LOG_FILE.json
```

Example prompt:

```text
First read problem.md and notes.tex, then list the key lemmas and gaps on which
the current proof depends. If necessary, write a small program to run numerical
or symbolic experiments, but clearly distinguish "experimental evidence" from
"rigorous proof." Finally, write the review report to review.md.
```

By default, every shell command requires confirmation. When AIM asks to run a command:

- enter `y` to approve that command only;
- enter `n`, or press Enter, to reject it;
- enter `a` to approve all subsequent commands for the remainder of the run.

You can also enable `--auto` at startup, but only do so when you trust both the current task and the contents of the workspace:

```bash
aimv2 --model gpt-5.6-sol --reasoning-effort high --enable-shell --auto
```

The shell tool allows its working directory to be set only within the current workspace, but this is **not a complete security sandbox**: commands still run with your local user permissions. They can modify or delete files and may access locations outside the workspace through absolute paths. We recommend retaining per-command confirmation and backing up important projects with Git or another method first.

### Use Case 4: Resume After an Interrupted Run

If you specified a log file when starting the session, run:

```bash
aimv2 resume --log-path YOUR_LOG_FILE.json
```

If you did not specify a log file, resume the most recent session from its original workspace:

```bash
aimv2 resume --last
```

Without `--last`, AIMv2 lists the sessions associated with the current workspace and lets you choose one:

```bash
aimv2 resume
```

After resuming and entering the session, you can run:

```text
/continue
```

`resume` and `/continue` serve different purposes:

- `resume` loads an existing session from disk;
- `/continue` retries the previous task **within an already open session** without adding a new user message.

### Use Case 5: Export All Intermediate Statements to Markdown

There is no need to inspect or edit the JSON manually. Use `view` instead:

```bash
aimv2 view --log-path YOUR_LOG_FILE.json --all > theorem-graph.md
```

To inspect and export the most recent session:

```bash
aimv2 view --last --all > theorem-graph.md
```

To inspect a single entry:

```bash
aimv2 view --log-path YOUR_LOG_FILE.json --id 12
```

To inspect an entry together with every dependency path leading to it:

```bash
aimv2 view --log-path YOUR_LOG_FILE.json --path-to 12 > theorem-path-12.md
```

`view` produces readable Markdown containing each statement, proof, dependency, review count, and reviewer comment. It only reads the log and does not require an API key.

### Use Case 6: Strengthen Proof Review

AIMv2 provides two reviewer modes:

- `progressive` (default): reviews the proof as a whole, then narrows its focus to progressively smaller proof segments if no issue is found;
- `simple`: runs several independent reviews in parallel.

To use the progressive reviewer:

```bash
aimv2 \
  --model gpt-5.6-sol \
  --reasoning-effort high \
  --reviewer progressive \
  --progressive-iterations 3 \
  --log-path YOUR_LOG_FILE.json
```

To run four simple reviews in parallel:

```bash
aimv2 \
  --model gpt-5.6-sol \
  --reasoning-effort high \
  --reviewer simple \
  --simple-reviews 4 \
  --log-path YOUR_LOG_FILE.json
```

Once inside the session, you can ask:

```text
Review the final statement and each entry along its dependency paths. Focus on
unstated assumptions, circular dependencies, changes in quantifiers, and steps
supported only by numerical experiments.
```

## Session Logs and Recovery Settings

Each session is stored in a separate JSON file. When starting a new session, we recommend always specifying the save location:

```bash
aimv2 --model gpt-5.6-sol --reasoning-effort high --log-path YOUR_LOG_FILE.json
```

This JSON file is the complete machine-readable record, including conversation messages, tool calls, session settings, and the theorem graph. The `view` command exports a readable Markdown version of the theorem graph; it is not a verbatim transcript of the conversation.

The value passed to `--log-path` must be a **file path**, not just a directory:

```bash
# Correct
aimv2 --model gpt-5.6-sol --reasoning-effort high --log-path aim-logs/session.json

# Incorrect: aim-logs is a directory
aimv2 --model gpt-5.6-sol --reasoning-effort high --log-path aim-logs
```

If `--log-path` is omitted, logs are still saved under an `aim-logs` directory in the operating system's temporary directory. The `history:` line printed at startup shows the full path. The system may clear temporary directories, so use an explicit log path for important tasks.

### Which Settings Are Preserved When You Resume a Session?

When resuming a log created by a recent version of AIMv2, the application restores the session settings saved in that log, including:

- the model and reasoning effort;
- the API endpoint (the API key is still read from the current environment);
- the token limit;
- the reviewer mode and review count;
- whether the shell is enabled and whether commands are approved automatically.

You should therefore decide whether to use `--enable-shell` **when creating a new session**. If the original session was created without shell access, you cannot temporarily enable it with a resume argument; create a new shell-enabled session instead. A command such as the following will also fail immediately because `--enable-shell` appears after the subcommand:

```bash
# Do not use this
aimv2 resume --enable-shell
```

If you only need to inspect or export an existing theorem graph, shell access is unnecessary; use `aimv2 view` directly.

## What Is the Theorem Graph?

AIMv2 maintains a dependency graph in the session log. It primarily contains two types of entries:

- `context`: assumptions and background obtained from the user, files, or other sources;
- `theorem`: statements and intermediate results formulated or derived by AIM during the current investigation.

Each entry can contain:

- a statement;
- a proof or supporting justification;
- the IDs of entries on which it depends;
- entries derived from it;
- the number of completed reviews;
- issues found by reviewers.

The graph is intended to make the dependencies and gaps in long proofs easier to inspect. It is not a list of literature search results. Even if an entry's type is `theorem`, assess its reliability by examining the proof and reviewer comments.

## Interactive Commands

| Command | Description |
| --- | --- |
| `/help` | Show help and the current session settings |
| `/continue` | Retry the previous task without adding a new user message |
| `/compact` | Manually compact a long context |
| `/reviewer simple` | Switch to the simple reviewer |
| `/reviewer progressive` | Switch to the progressive reviewer |
| `/reviews N` | Set the number of parallel simple reviews |
| `/iterations N` | Set the maximum number of progressive reviewer iterations |
| `/auto on` | Enable automatic approval when the shell is active |
| `/auto off` | Restore per-command confirmation when the shell is active |
| `/exit` or `/quit` | Exit the current session |

`Ctrl-C` cancels the current input line. `Ctrl-D` exits the session.

## Agent Skills

When `--enable-shell` is active, AIMv2 scans the following locations for skill directories that directly contain a `SKILL.md` file:

- `<workspace>/.aim/skills/`
- `<workspace>/.agents/skills/`
- `~/.aim/skills/`
- `~/.agents/skills/`

You can also add one or more additional directories:

```bash
aimv2 --reasoning-effort high --enable-shell --external-skills /path/to/team-skills

aimv2 \
  --model gpt-5.6-sol \
  --reasoning-effort high \
  --enable-shell \
  --external-skills /path/to/team-skills \
  --external-skills /path/to/private-skills
```

Skills are not loaded unless the shell is enabled, because AIM needs shell access to read the corresponding `SKILL.md` files.

## Troubleshooting

### `missing AIM_API_KEY or OPENAI_API_KEY`

No API key was found in the current workspace. Check that:

1. `.env` is in the directory from which you run `aimv2`;
2. the variable is named `AIM_API_KEY` or `OPENAI_API_KEY`;
3. there are no extra characters around the equals sign and the value is not empty.

### `model_not_found` or “No channel available for model”

This usually indicates that the current endpoint does not offer the selected model; it is not a theorem-graph or logging problem. If the error mentions the default model, `gpt-5.6-sol`, switch to a model your provider supports:

```bash
aimv2 --model exact-model-name-from-your-provider
```

Also verify that `AIM_BASE_URL` points to the correct endpoint and, if applicable, that the URL includes `/v1`.

### `failed to receive streaming response chunk`

First read the API error body that follows this message. If it also contains `model_not_found`, correct the model name first. If there is no specific API error, check your network connection, endpoint configuration, and provider status.

### `failed to read session log ... Is a directory (os error 21)`

You passed a directory instead of a specific session file. Use:

```bash
aimv2 resume --log-path YOUR_LOG_FILE.json
```

### `unexpected argument '--enable-shell' found`

The `resume` subcommand does not accept this option after the subcommand, and recent session logs preserve the original shell setting. If the original session did not have shell access, create a new session instead:

```bash
aimv2 --enable-shell --log-path YOUR_LOG_FILE.json
```

### `/continue` Does Not Load a Previous Session

This is expected. First load the log with `aimv2 resume ...`, then enter `/continue` from within the resumed session.

### `resume --last` Reports No History for the Current Workspace

Automatic discovery only matches the workspace path from which the session was originally created. Return to that directory, or specify the log file directly:

```bash
aimv2 resume --log-path YOUR_LOG_FILE.json
```

### How Can I See Which Statements Are in the JSON File?

You do not need to inspect the JSON directly:

```bash
aimv2 view --log-path YOUR_LOG_FILE.json --all > theorem-graph.md
```

### AIM Reports That a Large Write Command Was Rejected

This usually means that a shell command was not approved or failed to execute; it does not mean the session log is corrupted. Confirm that the startup information shows the `shell tool` as `enabled`, approve reasonable write commands, or ask AIM to split the content into smaller writes and try again. If the session was created without shell access, use `view` to export the theorem graph or create a new session with shell access enabled.

## Complete Command Reference

```text
# Start a new session
aimv2 [OPTIONS]

# Resume a session
aimv2 resume [--last] [--log-path FILE]

# Inspect the theorem graph
aimv2 view (--last | --log-path FILE) (--all | --id N | --path-to N)
```

Common startup options:

| Option | Default | Description |
| --- | --- | --- |
| `--model MODEL` | `gpt-5.6-sol` | Model name supported by the API provider |
| `--reasoning-effort LEVEL` | `medium` | `minimal` / `low` / `medium` / `high` |
| `--log-path FILE` | System temporary directory | Path to the session JSON file |
| `--enable-shell` | Disabled | Allow AIM to read and write files and run commands in the workspace |
| `--auto` | Disabled | Automatically approve commands when the shell is enabled |
| `--reviewer KIND` | `progressive` | `simple` or `progressive` |
| `--simple-reviews N` | `4` | Number of parallel reviews run by the simple reviewer |
| `--progressive-iterations N` | `3` | Maximum number of progressive reviewer iterations |
| `--token-limit N` | Model-dependent | Override the automatic context-compaction threshold |
| `--external-skills DIR` | None | Add a skill directory; may be specified more than once |

Consult the help output for the installed version on your machine:

```bash
aimv2 --help
aimv2 resume --help
aimv2 view --help
```

## Development and Verification

From the repository root:

```bash
cargo check
cargo test
cargo fmt --check
```

You can also run AIMv2 directly during development without installing it:

```bash
cargo run -- --help
cargo run -- --model gpt-5.6-sol
```

Background reviewers and automatic context compaction run silently. Only the main session displays normal assistant responses and tool activity.
