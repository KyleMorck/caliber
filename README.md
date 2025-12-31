# Caliber

A simple, bullet journal-inspired way to capture tasks and notes while you work in the terminal.

![caliber screenshot](screenshot.png)

## Installation

```bash
cargo install caliber
```

Requires Rust 1.85+ (edition 2024).

## Usage

```bash
caliber              # Open journal
caliber /path/to.md  # Use specific journal file
caliber init         # Create config file
```

### Daily View

| Key | Action |
|-----|--------|
| `Enter` | New entry at end |
| `i` | Insert entry below current |
| `e` | Edit selected entry |
| `j/k` | Navigate down/up |
| `x` | Toggle task complete |
| `d` | Delete entry |
| `h/l` or `[/]` | Previous/next day |
| `t` | Go to today |
| `g` | Gather completed tasks to top |
| `o` | Order mode (rearrange entries) |
| `Tab` | Switch to Tasks view |
| `?` | Show help |
| `:goto` or `:gt` | Go to date (YYYY/MM/DD or MM/DD) |
| `:q` | Quit |

### Order Mode

Rearrange entries within a day.

| Key | Action |
|-----|--------|
| `j/k` | Move entry up/down |
| `o` or `Enter` | Save new order |
| `Esc` | Cancel |

### Edit Mode

| Key | Action |
|-----|--------|
| `Enter` | Save and exit |
| `Tab` | Save and add new entry below |
| `⇧Tab` | Toggle entry type (task/note/event) |
| `←/→` | Move cursor |
| `Esc` | Cancel and return to Daily |

### Tasks View

Shows incomplete tasks across all days.

| Key | Action |
|-----|--------|
| `j/k` | Move down/up |
| `x` | Toggle task |
| `r` | Refresh list |
| `Enter` | Go to selected day |
| `Tab` | Return to Daily |
| `?` | Show help |

## Journal Format

Caliber uses a markdown-compatible format:

```markdown
# 2025/01/15
- [ ] Incomplete task
- [x] Completed task
- A note
* An event
```

## Configuration

Config file: `~/.config/caliber/config.toml`

```toml
# Custom default journal path
default_file = "/path/to/journal.md"
```

Default journal location: `~/.config/caliber/journals/journal.md`

## License

MIT
