# agent-browser

Browser automation CLI for AI agents with state mapping capabilities.

**Fork of [vercel-labs/agent-browser](https://github.com/vercel-labs/agent-browser)** with added state mapping feature for recording and analyzing web UI workflows.

## What's New: State Mapper

The mapper records browser interactions into a **state graph** that maps structurally similar pages to the same state, enabling efficient workflow analysis and AI-powered automation.

### Key Features

- **Pure Recording System**: No LLM built-in, you control the browser
- **Smart State Deduplication**: Similar pages (e.g., different GitHub repos) map to the same state
- **Rich Selector Info**: Captures ARIA attributes, roles, element tags, and classes
- **Structural Fingerprinting**: Pages with same UI structure but different content share state IDs
- **Task Organization**: Mark and track workflows as named tasks

### Quick Example

```bash
# Start recording
agent-browser map start --site "github.com"

# Navigate and interact normally
agent-browser open https://github.com/rust-lang/rust
agent-browser snapshot -i  # See @e refs
agent-browser click "@e28"  # Click Issues link
agent-browser click "@e93"  # Click first issue

# Stop and save the graph
agent-browser map stop github-map.json
```

**Output**: JSON with nodes (UI states) and edges (actions), with aggressive generalization:
- All GitHub repo pages → same state
- All GitHub issue list pages → same state
- All Reddit subreddit pages → same state

### How It Works

**State Graph**:
- **Nodes** = Unique UI states (identified by URL pattern + element structure)
- **Edges** = Actions with rich selector info (ARIA, role, name, tag, class)

**State Hashing**: Combines URL pattern (e.g., `github:repo`) with bucketed element counts:
```
Structure: button×20, link×25, heading×5 → bucket: button:11-20, link:21-50, heading:2-5
Hash: SHA-256(url_pattern + canonical_structure) → State ID
```

Different content, same structure = same state!

### Mapper Commands

```bash
# Start recording
agent-browser map start --site "example.com"

# Mark tasks (optional)
agent-browser map task "workflow_name"          # Start task
agent-browser map task "workflow_name" --end    # End task

# Stop and save
agent-browser map stop output.json
```

### Use Cases

1. **Manual Mapping**: Record workflows once, let AI replay them
2. **LLM-Driven Mapping**: External scripts use LLM to drive agent-browser while recording
3. **UI Change Detection**: Compare maps over time to detect structural changes
4. **Workflow Documentation**: Auto-generate state graphs for testing/debugging

### Output Format

```json
{
  "site": "github.com",
  "generated_at": "2026-03-31T09:06:54Z",
  "nodes": {
    "state_id_1": {
      "id": "state_id_1",
      "url": "https://github.com/rust-lang/rust",
      "snapshot": "- link \"Issues\" [ref=e28]\n...",
      "title": "rust-lang/rust"
    }
  },
  "edges": [
    {
      "id": "e1",
      "from": "state_id_1",
      "to": "state_id_2",
      "selector": {
        "raw": "@e28",
        "aria": "role=link name=\"Issues 5k+\"",
        "name": "Issues 5k+",
        "role": "link"
      },
      "element": {
        "tag": "a",
        "class": "nav-link"
      },
      "action_type": "click",
      "description": "click"
    }
  ],
  "tasks": []
}
```

---

## Installation

### Global Installation (recommended)

```bash
npm install -g agent-browser
agent-browser install  # Download Chrome from Chrome for Testing
```

### From Source

```bash
git clone https://github.com/YashJain14/agent-browser-mapper
cd agent-browser-mapper
pnpm install
pnpm build
pnpm build:native   # Requires Rust (https://rustup.rs)
pnpm link --global
agent-browser install
```

### Requirements

- **Chrome** - Run `agent-browser install` to download Chrome from [Chrome for Testing](https://developer.chrome.com/blog/chrome-for-testing/)
- **Rust** - Only needed when building from source

---

## Quick Start (Core Features)

```bash
agent-browser open example.com
agent-browser snapshot                    # Get accessibility tree with refs
agent-browser click @e2                   # Click by ref from snapshot
agent-browser fill @e3 "test@example.com" # Fill by ref
agent-browser get text @e1                # Get text by ref
agent-browser screenshot page.png
agent-browser close
```

---

## Core Commands

### Navigation

```bash
agent-browser open <url>              # Navigate to URL
agent-browser back                    # Go back
agent-browser forward                 # Go forward
agent-browser reload                  # Reload page
agent-browser close                   # Close browser
```

### Inspection

```bash
agent-browser snapshot                # Get accessibility tree
agent-browser snapshot -i             # Interactive mode with refs (@e1, @e2, ...)
agent-browser snapshot --json         # JSON output
agent-browser get url                 # Get current URL
agent-browser get title               # Get page title
agent-browser get text <selector>     # Get element text
agent-browser get html <selector>     # Get element HTML
```

### Interaction

```bash
agent-browser click <selector>        # Click element
agent-browser fill <selector> "text"  # Fill input
agent-browser press Enter             # Press keyboard key
agent-browser hover <selector>        # Hover over element
agent-browser drag @e1 @e2            # Drag and drop
```

### Screenshots

```bash
agent-browser screenshot output.png                  # Full page
agent-browser screenshot --selector @e1 element.png  # Specific element
agent-browser screenshot --annotate output.png       # With element highlights
```

### State Management

```bash
agent-browser save session.json       # Save cookies, localStorage
agent-browser load session.json       # Restore session state
```

### Advanced

```bash
agent-browser trace                   # Start recording HAR trace
agent-browser trace stop trace.har    # Stop and save trace
agent-browser stream start            # Start live preview server
agent-browser pdf output.pdf          # Save page as PDF
```

---

## Options

### Global Options

```
--headless           Run in headless mode (no visible window)
--no-headless        Run in headed mode (visible window)
--user-data-dir <path>   Persist browser data (cookies, cache, extensions)
--window-size <WxH>  Set window size (default: 1280x720)
--timeout <ms>       Action timeout in milliseconds (default: 25000)
--engine <engine>    Browser engine: chrome (default), lightpanda
```

### Selector Types

- **Refs**: `@e1`, `@e2` (from `snapshot -i`)
- **CSS**: `#id`, `.class`, `button.primary`
- **XPath**: `//button[@id='submit']`
- **Text**: `text="Submit"`, `text*="Submit"` (contains)
- **ARIA**: `role=button`, `role=link name="Submit"`

---

## Architecture

- **CLI**: Parses commands, communicates with daemon
- **Daemon**: Long-running process managing Chrome via CDP (Chrome DevTools Protocol)
- **Mapper**: Records state transitions into graph with structural deduplication
- **Fast**: Native Rust, <50ms command overhead

---

## Building

```bash
pnpm build                # Build dashboard
pnpm build:native         # Build for current platform
pnpm build:all-platforms  # Build for all 7 platforms (Docker required)
```

### Testing

```bash
cd cli && cargo test                              # Unit tests
cd cli && cargo test e2e -- --ignored --test-threads=1  # E2E tests
```

---

## Original Project

This is a fork of [vercel-labs/agent-browser](https://github.com/vercel-labs/agent-browser). See the original repository for full documentation of core features.

### Changes in This Fork

- Added state mapping system (`map start`, `map stop`, `map task`)
- Structural state fingerprinting with aggressive generalization
- Rich selector information capture (ARIA, role, name, tag, class, id)
- JSON output with state graph (nodes, edges, tasks)

---

## License

MIT - See LICENSE file for details
