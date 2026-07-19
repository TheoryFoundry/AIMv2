# AIMv2

[English](README.md)

`aimv2` 是一个在本地工作目录中运行的 AI 数学助手（命令行工具）。它支持：

- 探索数学问题与证明思路；
- 把中间命题、证明及其依赖关系保存为 theorem graph；
- 对证明做多轮审查并记录问题；
- 在长任务中保存进度，之后继续工作；
- 在获得授权后读取本地材料、运行代码和写入结果文件。

> **术语说明：** theorem graph 里的 “theorem” 主要指 AIM 在解题过程中提出或推导出的 **statement（命题、中间结论）**，不一定是从文献中检索到的已有定理。

## 5 分钟快速开始

### 1. 准备 Rust

安装 AIMv2 需要 Rust 和 Cargo。先确认本机可以执行：

```bash
rustc --version
cargo --version
```

如果命令不存在，请先从 [Rust 官方安装页面](https://www.rust-lang.org/tools/install) 安装 Rust。已安装 [Homebrew](https://brew.sh/) 的 macOS 用户也可以直接执行：

```bash
brew install rust
```

### 2. 安装 AIMv2

进入本仓库根目录，执行：

```bash
cargo install --path .
```

安装完成后检查：

```bash
aimv2 --help
```

以后可以在任意数学项目目录中直接运行 `aimv2`。更新本地源码后，可用下面的命令覆盖安装：

```bash
cargo install --path . --force
```

### 3. 创建工作目录并配置 API

AIMv2 会把**启动命令时所在的目录**当作 workspace。建议每个问题或项目使用独立目录：

```bash
mkdir my-math-project
cd my-math-project
```

在该目录中新建 `.env`：

```dotenv
AIM_API_KEY=请替换为你的密钥
AIM_BASE_URL=https://请替换为服务商地址/v1
```

例如，若服务商提供的 endpoint 是 `https://endpoint.com/v1`，则写成：

```dotenv
AIM_API_KEY=请替换为你的密钥
AIM_BASE_URL=https://endpoint.com/v1
```

如果直接使用 OpenAI 官方 API，只需配置官方 API key，无需设置 endpoint：

```dotenv
OPENAI_API_KEY=请替换为你的密钥
```

此时 AIMv2 会使用默认 endpoint `https://api.openai.com/v1`。可以在 [OpenAI API Keys](https://platform.openai.com/settings/organization/api-keys) 页面创建或管理密钥。

不要把真实 API key 提交到 Git。请确认项目的 `.gitignore` 中包含 `.env`。

### 4. 第一次运行

模型名必须是 API 服务商实际提供的名称。下面以 `gpt-5.6-sol` 为例：

```bash
# 1. 请按需调整 log 存放目录和文件；
# 2. 不建议放在工作目录下，以免 aimv2 自动读取产生混乱
# 3. 如不指定 --log-path，log 将存放在系统默认目录

aimv2 \
  --model gpt-5.6-sol \
  --reasoning-effort high \
  --enable-shell \
  --log-path YOUR_LOG_FILE.json
```

> **安全提示：** `--enable-shell` 允许 AIM 以当前用户权限执行命令和修改文件，并不提供完整的安全沙箱。首次使用建议保留默认的逐条确认模式，不要添加 `--auto`；如果任务不需要读取文件、运行代码或写入结果，请移除 `--enable-shell`。

启动信息中请检查以下几项：

- `workspace`：是否是刚才创建的项目目录；
- `model`：是否是服务商支持的模型；
- `reasoning effort`：是否是需要的推理强度；
- `shell tool`：是否符合预期；
- `history`：本次会话日志实际保存在哪里。

然后直接用自然语言输入问题。例如：

```text
请研究下面的问题。先明确假设与目标，再探索证明路线；把可靠的中间命题及依赖关系记录到 theorem graph 中：

设 f:[0,1]→R 连续且 ∫₀¹f(x)dx=0。证明存在 ξ∈(0,1)，使得……
```

输入 `/help` 可以查看会话内命令，输入 `/exit` 结束会话。

如果希望 progressive reviewer 做更深入的分段审查，可以在会话内调高最大迭代次数，例如：

```text
/iterations 5
```

迭代次数越多，通常 API 调用次数越多、等待时间越长、费用也越高，但这并不能保证证明一定正确。

## API 与模型配置

### 环境变量

AIMv2 按以下优先级读取配置：

| 用途 | 优先读取 | 备用变量 | 未设置时 |
| --- | --- | --- | --- |
| API key | `AIM_API_KEY` | `OPENAI_API_KEY` | 无法启动 |
| API endpoint | `AIM_BASE_URL` | `OPENAI_BASE_URL` | `https://api.openai.com/v1` |

推荐把配置写在 workspace 根目录的 `.env` 中。AIMv2 只会自动读取**当前 workspace 下的 `.env`**；安装源码目录中的 `.env` 不会自动应用到其他 workspace。

也可以只为当前终端临时设置：

```bash
export AIM_API_KEY='请替换为你的密钥'
export AIM_BASE_URL='https://请替换为服务商地址/v1'
```

### 选择模型

AIMv2 当前默认模型是 `gpt-5.6-sol`，但第三方 endpoint 不一定提供它。最稳妥的做法是每次新建会话时明确指定服务商支持的模型：

```bash
aimv2 --model gpt-5.6-sol
```

`gpt-5.6-sol` 只是示例；请以你的服务商模型列表为准。模型标识通常需要完全一致。

还可以调整推理强度：

```bash
aimv2 --model gpt-5.6-sol --reasoning-effort high
```

可选值为 `minimal`、`low`、`medium`、`high`，默认是 `medium`。

## 推荐的项目结构

```text
my-math-project/
├── .env                    # API 配置，不要提交到 Git
├── .gitignore
├── problem.md              # 问题描述、定义和约定
└── notes.tex               # 可选：已有证明或草稿
```

请先 `cd` 到正确的项目目录再启动 AIMv2。workspace 决定了：

- 自动读取哪个 `.env`；
- `--enable-shell` 后 AIM 被要求在哪个目录内读写和运行命令；
- 相对日志路径保存到哪里；
- `resume --last` 和 `view --last` 会查找哪些历史会话。

## 典型使用场景

### 场景一：从一个明确的问题开始探索

```bash
# 1. 请按需调整 log 存放目录和文件；
# 2. 不建议放在工作目录下，以免 aimv2 自动读取产生混乱
# 3. 如不指定 --log-path，log 将存放在系统默认目录

cd my-math-project
aimv2 --model gpt-5.6-sol --reasoning-effort high --log-path YOUR_LOG_FILE.json
```

示例提问：

```text
请帮助我探究一下我正在研究的各个问题，先检查命题是否可能成立，再逐步给出证明或证伪方法。
```

这种场景不需要读写本地文件时，可以不启用 shell。

### 场景二：问题还比较模糊，先帮助定题

```bash
# 1. 请按需调整 log 存放目录和文件；
# 2. 不建议放在工作目录下，以免 aimv2 自动读取产生混乱
# 3. 如不指定 --log-path，log 将存放在系统默认目录

aimv2 --model gpt-5.6-sol --reasoning-effort high --enable-shell --log-path YOUR_LOG_FILE.json
```

示例提问：

```text
我想研究“神经网络是否能发现组合数学中的新不变量”，但问题还不够具体。请先帮我澄清研究对象、允许的假设、可验证目标和可能的反例，给出 3 个由弱到强的候选问题；暂时不要直接声称已经证明。
```

本仓库包含 `.aim/skills/problem-clarifier/SKILL.md`，用于辅助把模糊想法整理为明确问题。通过 `cargo install` 安装二进制不会把仓库中的技能自动复制到其他 workspace；如需在自己的项目中使用，可以先执行：

```bash
mkdir -p .aim/skills
cp -R /path/to/AIMv2/.aim/skills/problem-clarifier .aim/skills/ 
```

请把 `/path/to/AIMv2` 替换为本仓库的实际路径。技能只会在启用 `--enable-shell` 时被发现。

### 场景三：结合已有 LaTeX、Markdown 或代码材料

```bash
# 1. 请按需调整 log 存放目录和文件；
# 2. 不建议放在工作目录下，以免 aimv2 自动读取产生混乱
# 3. 如不指定 --log-path，log 将存放在系统默认目录

cd my-math-project
aimv2 \
  --model gpt-5.6-sol \
  --reasoning-effort high \
  --enable-shell \
  --log-path YOUR_LOG_FILE.json
```

示例提问：

```text
请先阅读 problem.md 和 notes.tex，列出当前证明依赖的关键引理与缺口。必要时可以编写小程序做数值或符号实验，但请区分“实验支持”和“严格证明”。最后把审查报告写入 review.md。
```

shell 默认采用逐条确认模式。AIM 请求执行命令时：

- 输入 `y`：只批准本次命令；
- 输入 `n` 或直接回车：拒绝；
- 输入 `a`：本次运行后续命令自动批准。

也可以在启动时使用 `--auto`，但只应在你信任当前任务与 workspace 内容时使用：

```bash
aimv2 --model gpt-5.6-sol --reasoning-effort high --enable-shell --auto
```

shell 仅允许把工具的工作目录设在当前 workspace 内，但这**不是完整的安全沙箱**：实际命令仍具有本机用户的权限。它可以修改或删除文件，也可能通过绝对路径访问 workspace 之外的位置。建议保留逐条确认；重要项目请先使用 Git 或其他方式备份。

### 场景四：进程中断后继续任务

若启动时指定了日志文件：

```bash
aimv2 resume --log-path YOUR_LOG_FILE.json
```

若未指定日志文件，可在原 workspace 中恢复最近一次会话：

```bash
aimv2 resume --last
```

不加 `--last` 时，AIMv2 会列出当前 workspace 的历史会话供选择：

```bash
aimv2 resume
```

恢复并进入会话后，可输入：

```text
/continue
```

`resume` 和 `/continue` 的作用不同：

- `resume`：从磁盘加载一份已有会话；
- `/continue`：在**已经打开的会话中**，不添加新用户消息，重试上一次任务。

### 场景五：导出所有中间命题为 Markdown

不需要手工阅读或修改 JSON，可以直接使用 `view`：

```bash
aimv2 view --log-path YOUR_LOG_FILE.json --all > theorem-graph.md
```

查看最近会话并导出：

```bash
aimv2 view --last --all > theorem-graph.md
```

查看单个条目：

```bash
aimv2 view --log-path YOUR_LOG_FILE.json --id 12
```

查看某个条目及其全部依赖路径：

```bash
aimv2 view --log-path YOUR_LOG_FILE.json --path-to 12 > theorem-path-12.md
```

`view` 输出的是便于阅读的 Markdown，包含 statement、proof、依赖、审查次数和审查意见。它只读取日志，不需要 API key。

### 场景六：加强证明审查

AIMv2 有两种 reviewer：

- `progressive`（默认）：先整体审查；若未发现问题，再逐步缩小到证明片段；
- `simple`：并行执行若干次相互独立的审查。

使用 progressive reviewer：

```bash
aimv2 \
  --model gpt-5.6-sol \
  --reasoning-effort high \
  --reviewer progressive \
  --progressive-iterations 3 \
  --log-path YOUR_LOG_FILE.json
```

使用 4 次并行 simple review：

```bash
aimv2 \
  --model gpt-5.6-sol \
  --reasoning-effort high \
  --reviewer simple \
  --simple-reviews 4 \
  --log-path YOUR_LOG_FILE.json
```

进入会话后可以直接要求：

```text
请对最终命题及其依赖路径逐项审查。重点寻找未声明的假设、循环依赖、量词变化和只被数值实验支持的步骤。
```

## 会话日志与恢复设置

每个 session 对应一个独立的 JSON 文件。建议新建会话时总是明确保存位置：

```bash
aimv2 --model gpt-5.6-sol --reasoning-effort high --log-path YOUR_LOG_FILE.json
```

这个 JSON 是完整的机器可读记录，包含对话消息、工具调用、会话设置和 theorem graph。`view` 命令导出的是 theorem graph 的易读 Markdown，并不是逐字对话稿。

`--log-path` 后面必须是**文件路径**，不能只是目录：

```bash
# 正确
aimv2 --model gpt-5.6-sol --reasoning-effort high --log-path aim-logs/session.json

# 错误：aim-logs 是目录
aimv2 --model gpt-5.6-sol --reasoning-effort high --log-path aim-logs
```

如果不指定，日志仍会保存在操作系统临时目录下的 `aim-logs` 目录中；每次启动时终端的 `history:` 行会显示完整路径。临时目录可能被系统清理，重要任务应使用显式日志路径。

### 恢复会话时哪些设置会被继承？

恢复较新版本生成的日志时，AIMv2 会采用该日志中保存的会话设置，包括：

- 模型和 reasoning effort；
- API endpoint（API key 仍从当前环境读取）；
- token limit；
- reviewer 类型与次数；
- shell 是否启用以及是否自动批准。

因此，`--enable-shell` 应在**新建会话时**决定。如果原会话创建时没有启用 shell，恢复后不能用命令行参数临时打开；建议新建一个启用了 shell 的 session。类似下面的命令还会因为 `--enable-shell` 放在子命令之后而直接报参数错误：

```bash
# 不要这样写
aimv2 resume --enable-shell
```

如果只是想查看或导出已有 theorem graph，不需要启用 shell，直接使用 `aimv2 view` 即可。

## Theorem graph 是什么？

AIMv2 会在会话日志中维护一张依赖图，主要包含两类条目：

- `context`：从用户、文件或其他来源获得的前提与背景；
- `theorem`：AIM 在当前探索过程中提出或推导的命题与中间结论。

每个条目可以包含：

- statement；
- proof 或依据；
- 所依赖条目的 ID；
- 由它导出的条目；
- 已完成的审查次数；
- reviewer 发现的问题。

这张图的目标是让长证明的依赖和缺口更容易检查。它不是文献检索结果列表；即使条目类型叫 `theorem`，仍应根据 proof 和 reviewer comments 判断其可靠性。

## 会话内命令

| 命令 | 作用 |
| --- | --- |
| `/help` | 显示帮助和当前会话设置 |
| `/continue` | 不添加新用户消息，重试上一任务 |
| `/compact` | 手动压缩较长的上下文 |
| `/reviewer simple` | 切换为 simple reviewer |
| `/reviewer progressive` | 切换为 progressive reviewer |
| `/reviews N` | 设置 simple reviewer 的并行审查次数 |
| `/iterations N` | 设置 progressive reviewer 的最大迭代次数 |
| `/auto on` | shell 已启用时，打开自动批准 |
| `/auto off` | shell 已启用时，恢复逐条确认 |
| `/exit` 或 `/quit` | 退出当前会话 |

`Ctrl-C` 取消当前输入行，`Ctrl-D` 退出会话。

## Agent Skills

启用 `--enable-shell` 后，AIMv2 会扫描以下目录中直接包含 `SKILL.md` 的技能文件夹：

- `<workspace>/.aim/skills/`
- `<workspace>/.agents/skills/`
- `~/.aim/skills/`
- `~/.agents/skills/`

也可以添加一个或多个额外目录：

```bash
aimv2 --reasoning-effort high --enable-shell --external-skills /path/to/team-skills

aimv2 \
  --model gpt-5.6-sol \
  --reasoning-effort high \
  --enable-shell \
  --external-skills /path/to/team-skills \
  --external-skills /path/to/private-skills
```

未启用 shell 时不会加载 skills，因为 AIM 需要通过 shell 读取相应的 `SKILL.md`。

## 常见问题排查

### `missing AIM_API_KEY or OPENAI_API_KEY`

当前 workspace 中没有读到 API key。确认：

1. `.env` 位于执行 `aimv2` 时的当前目录；
2. 变量名是 `AIM_API_KEY` 或 `OPENAI_API_KEY`；
3. 等号两侧没有多余字符，值不是空的。

### `model_not_found` 或 “模型无可用渠道”

这通常不是 theorem graph 或日志问题，而是当前 endpoint 不提供所选模型。若错误中出现默认模型 `gpt-5.6-sol`，请改为服务商支持的模型：

```bash
aimv2 --model 服务商提供的准确模型名
```

同时检查 `AIM_BASE_URL` 是否是正确 endpoint，必要时确认是否需要 `/v1`。

### `failed to receive streaming response chunk`

先阅读它后面的 API 错误正文。如果正文同时包含 `model_not_found`，应优先修正模型名；如果没有明确 API 错误，再检查网络、endpoint 和服务状态。

### `failed to read session log ... Is a directory (os error 21)`

传入的是目录，不是单个 session 文件。改为：

```bash
aimv2 resume --log-path YOUR_LOG_FILE.json
```

### `unexpected argument '--enable-shell' found`

`resume` 不接受在子命令后追加这个选项，而且恢复的新式日志会继承原会话的 shell 设置。若原会话没有 shell，推荐新建：

```bash
aimv2 --enable-shell --log-path YOUR_LOG_FILE.json
```

### `/continue` 没有加载以前的会话

这是预期行为。先用 `aimv2 resume ...` 加载日志，进入会话后再输入 `/continue`。

### `resume --last` 显示当前 workspace 没有历史会话

自动发现只匹配创建会话时的 workspace 路径。请回到原目录运行，或直接指定日志文件：

```bash
aimv2 resume --log-path YOUR_LOG_FILE.json
```

### 如何查看 JSON 中有哪些命题？

无需直接阅读 JSON：

```bash
aimv2 view --log-path YOUR_LOG_FILE.json --all > theorem-graph.md
```

### AIM 提示较大的写入命令被拒绝

这通常表示某次 shell 命令未获批准或执行失败，不代表 session 日志损坏。请确认启动信息中的 `shell tool` 是 `enabled`，批准合理的写入命令，或要求 AIM 把内容拆成较小的写入步骤后重试。若该会话创建时没有启用 shell，请使用 `view` 导出 theorem graph，或新建一个启用了 shell 的 session。

## 完整命令速查

```text
# 新建会话
aimv2 [OPTIONS]

# 恢复会话
aimv2 resume [--last] [--log-path FILE]

# 查看 theorem graph
aimv2 view (--last | --log-path FILE) (--all | --id N | --path-to N)
```

常用启动选项：

| 选项 | 默认值 | 说明 |
| --- | --- | --- |
| `--model MODEL` | `gpt-5.6-sol` | API 服务商支持的模型名 |
| `--reasoning-effort LEVEL` | `medium` | `minimal` / `low` / `medium` / `high` |
| `--log-path FILE` | 系统临时目录 | session JSON 文件路径 |
| `--enable-shell` | 关闭 | 允许 AIM 在 workspace 内读写文件和运行命令 |
| `--auto` | 关闭 | shell 启用时自动批准命令 |
| `--reviewer KIND` | `progressive` | `simple` 或 `progressive` |
| `--simple-reviews N` | `4` | simple reviewer 的并行审查次数 |
| `--progressive-iterations N` | `3` | progressive reviewer 的最大迭代次数 |
| `--token-limit N` | 按模型选择 | 覆盖自动上下文压缩阈值 |
| `--external-skills DIR` | 无 | 添加技能目录，可重复使用 |

以本机实际版本为准：

```bash
aimv2 --help
aimv2 resume --help
aimv2 view --help
```

## 开发与检查

在仓库根目录中：

```bash
cargo check
cargo test
cargo fmt --check
```

开发时不安装也可以直接运行：

```bash
cargo run -- --help
cargo run -- --model gpt-5.6-sol
```

后台 reviewer 和自动上下文压缩会静默运行；正常的助手回复和工具活动只由主会话显示。
