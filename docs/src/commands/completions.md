# mc completions

```text
mc completions <SHELL>
```

Prints a completion script for the given shell to standard output. Accepted
shells: `bash`, `elvish`, `fish`, `powershell`, `zsh`.

Install it wherever the shell loads completions from. For example:

```sh
mc completions fish > ~/.config/fish/completions/mc.fish
mc completions bash > ~/.local/share/bash-completion/completions/mc
mc completions zsh > ~/.zfunc/_mc
```

The script describes the commands and options of the binary that generated
it; regenerate it after upgrading mc.

## Examples

```sh
mc completions fish > ~/.config/fish/completions/mc.fish
```
