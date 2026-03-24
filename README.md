# dochub

Small CLI for managing named document directories from `~/.dochub/hub.toml`.

## Install

```bash
cargo install --path .
```

## Commands

```bash
dochub add <skill_name> <path>
dochub prune
dochub sanity
dochub cp <skill_name> <dest>
dochub use <skill_name> [dest] # dest defaults to .
dochub rm <skill_name>
dochub ls [skill_name]
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
