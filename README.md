# 🦞 Claw-FS: The Indestructible Text & Code CLI for AI Agents

Claw-FS is a specialized command-line tool designed to bridge the gap between Autonomous AI Agents (like OpenClaw, DeepSeek, Qwen, or Claude) and the local file system. 

When LLM agents use standard bash tools (like `cat >>`, `sed`, or generic `write_file` tools), they often hallucinate, truncate outputs when reaching their token limits, corrupt data with bad encodings, or destroy large files. **Claw-FS fixes this completely.**

## ✨ Why Claw-FS?

1. **JSON Output Only:** LLMs are native JSON machines. Claw-FS always responds with clean, structured JSON so the agent never gets confused by erratic bash outputs.
2. **Atomic Writes & Auto-Backups (`.bak`):** Every destructive operation (append, insert, replace) creates an instant atomic backup. If the agent breaks the code, they can instantly call `undo` to revert it.
3. **Line-Range Surgery:** Instead of searching and replacing generic text (which can replace the wrong variables), agents can replace specific chunks using precise line numbers (`start` and `end`).
4. **Token-Saving Reads:** The `outline` and `read-chunk` commands allow agents to navigate massive codebases (thousands of lines) without filling up their context window.

---

## 🚀 Installation

Claw-FS is provided in two identical flavors depending on your environment: **Rust** (The standard for Linux/Servers, blazing fast) and **Node.js** (For Windows or rapid prototyping).

### Option A: Rust (Recommended for Servers/Linux)
1. Ensure Rust is installed (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`).
2. Clone this repository and build:
```bash
git clone https://github.com/Miltondz/claw-fs.git
cd claw-fs
cargo build --release
cp target/release/claw-fs ~/.local/bin/claw-fs
```

### Option B: Node.js (Windows)
1. Ensure Node.js is installed.
2. No compilation needed. The repository includes `claw-fs.js` and a `claw-fs.cmd` wrapper.
3. Just add the repository folder to your Windows `%PATH%` or call `claw-fs.cmd` directly.

---

## 🛠️ Command Reference

All commands return a JSON payload: `{"status": "success|error", "message": "...", "data": {}}`

### 1. `append` - Safe Continuation
Appends text to the end of a file. Perfect for generating long stories or writing logs in chunks, bypassing LLM max-token limits.
```bash
claw-fs append --file path/to/story.md --content "Chapter 2 begins here..."
```
*Returns line and word count updates instantly.*

### 2. `insert` - Precise Injection
Injects text at a specific line number (`1-indexed`) without altering the rest of the file.
```bash
claw-fs insert --file app.js --line 45 --content "console.log('Injected safely');"
```

### 3. `replace-lines` - Context-Aware Replacement
The holy grail for code modification. Replaces an exact block of code between start and end lines.
```bash
claw-fs replace-lines --file server.js --start 100 --end 115 --content "function newSafeCode() { ... }"
```

### 4. `read-chunk` - Token-Saving Reading
Reads only a specific chunk of lines to save LLM context window memory.
```bash
claw-fs read-chunk --file giant_log.txt --start 500 --end 520
```

### 5. `find-refs` - Search with Context
Searches for a Regex query and returns matches wrapped in `N` lines of surrounding context.
```bash
claw-fs find-refs --file database.js --query "dbConnection" --context 2
```

### 6. `outline` - Codebase X-Ray
Parses the file structure and extracts ONLY the signatures of classes, functions, and global constants (JS/TS/PY targets). 
```bash
claw-fs outline --file index.js
```
*Returns a JSON array of definitions and their exact line numbers. The ultimate roadmap for an agent.*

### 7. `undo` - The Lifesaver
If an agent realizes their last edit broke the linting or logic, they just call undo. Reverts the file to the state before the last `append`, `insert`, or `replace-lines` operation.
```bash
claw-fs undo --file app.js
```

### 8. `stats` - File Health
Quick sanity check for the file's current size, line count, and word count.
```bash
claw-fs stats --file story.md
```

---

## 🤖 Prompt for Agents (How to strictly enforce Claw-FS)

To make your AI agent use this tool exclusively, paste this into their core prompt or chat:

> "From now on, when you need to read, modify, append text, or explore code in my projects, you MUST strictly use the universal tool `claw-fs`. Its core commands are: `claw-fs append --file <path> --content <text>`, `claw-fs replace-lines --file <path> --start <X> --end <Y> --content <text>`, `claw-fs read-chunk`, `claw-fs find-refs`, `claw-fs outline`, and `claw-fs undo`. This tool outputs structured JSON, prevents data corruption, and handles atomic backups. Use `claw-fs --help` if in doubt about syntax. Never use standard bash writes (cat, sed) again."

---
*🦞 Built by Milton & AntiGravity for the OpenClaw Ecosystem.*
