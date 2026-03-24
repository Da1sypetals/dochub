# dochub

Small CLI for managing named document directories from `~/.dochub/hub.toml`.

## Install

```bash
cargo install --path .
```

## Commands

```bash
dochub add <name> <path>
dochub prune
dochub sanity
dochub cp <name> <dest>
dochub skill cp <name> [dest] # dest defaults to .
dochub rm <name>
dochub ls [name]
```

## Config

Config lives at `~/.dochub/hub.toml`.

Example:

```toml
sane-size = 16 # in Megabytes
ignore = ["skipme", "*.tmp"]
skill-dir = [".claude/skill/", ".cursor/skill"]

[hub]
docs = "/path/to/docs"
```
