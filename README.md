# Single-line Version Control System

A simple version control system for lightweight use.

## Usage

```
$ svc -h

Usage: svc [COMMAND]

Commands:
  info        show repo info
  init        initialize a svc repo
  log         show all versions log
  status      check files status
  commit      save current workspace
  checkout    switch to specific version
  push        push to remote repo
  pull        pull from remote repo
  set-remote  set remote repo url
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Notice
- No index. `svc commit` will save the whole workspace(except for files declared in `.svcignore`).
- No branches. `svc commmit` will remove commits after HEAD when it's about to branch.

## WIP
- implement remote repo synchronization.
- optimized terminal interaction.
- waiting for bug and fix.
