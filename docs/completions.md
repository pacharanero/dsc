# Shell completions

Generate shell completion scripts for bash, zsh, or fish.

```
dsc completions <shell> [--dir <path>]
```

If `--dir` is provided, writes the completion script to the given directory. Otherwise, prints to stdout.

## Setup

```bash
# Bash
dsc completions bash --dir /usr/local/share/bash-completion/completions

# Zsh
dsc completions zsh --dir ~/.zsh/completions
echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc
autoload -Uz compinit && compinit

# Fish
dsc completions fish --dir ~/.config/fish/completions
```

The Zsh generator writes a `_dsc` file in the target directory. It includes dynamic completion of discourse names for positional arguments (calls `dsc list --format plaintext` at completion time).

Regenerate completions after upgrading `dsc` to pick up new commands and flags.
