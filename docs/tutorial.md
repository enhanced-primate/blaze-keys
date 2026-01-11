# Tutorial 

Before following this tutorial, follow the Quick Start steps in the main `README.md`, which will instruct you to download `blz`, modify the `~/.zshrc` and create the `all` config via `blz -g`. You can view/edit the global config by running `blz -g` at any point. 

Don't forget to `source ~/.zshrc` after the last step.

## Leader keys

### Modes

Leader keys can be triggered in two modes: `exec` and `abbr`. The trigger keys are defined in the global config: 

```yml
    - name: Leader1
      exec_mode: "Ctrl-s"   <--- keybind to trigger exec
      abbr_mode: "Alt-s"    <--- keybind to trigger abbr
      combos: |
        ...
```

- `exec` mode
  - In `exec` mode, the command will execute without confirmation as soon as it completes. You won't need to press Enter/Return. 
- `abbr` mode 
  - In `abbr` mode, the command will expand into the current line as soon as it completes. You will be left to add text or press Enter/Return. 

#### Try out `exec` mode

In the default config, the `exec` leader key is defined as `Ctrl-s`. Try running `git status` with this leader key:

1. Press `Ctrl-s`.
2. Press `g`, then press `s`.

You should find `git status` has executed in your shell. You could now try running it again, as quickly as you can. 

Some commands may have subcommands; in this case, you need to press `Space` to select the non-subcommand. Try running `git log`:

1. Press `Ctrl-s`.
2. Press `g`, then `l`, then `Space`.

You should find `git log` has executed. Similarly, `git log --oneline` is triggered by typing `glo` after the leader key, with no space required because it has no subcommands. 

#### Try out `abbr` mode

In the default config, the `abbr` leader key is defined as `Alt-s`. Try expanding `git checkout` with this leader key:

1. Press `Alt-s`.
2. Press `g`, then `c`. Because there are subcommands, also press `Space`. 

You should find that `git checkout` has expanded into your current line. You can now type any remaining parts and press Enter. 

#### Composing commands with multiple invocations

To get even faster, you can use multiple invocations of leader keys in one command. For example, you may often have to run `git checkout origin/main`. An effective way to optimise this is by putting `origin/main` behind a different leader key (or the same leader key, if you like). In the `all` template, `origin/main` is defined under leader key `Alt-o`.

To run `git checkout origin/main`, you can:

1. Press `Alt-s` to trigger `Leader1` in `exec` mode. 
2. Type `gc` and press `Space`. Now your current line should be `git checkout `. 
3. Press `Ctrl-o` to trigger `Leader0` in `exec` mode. This will execute the line after completing.
4. Type `om` to trigger `origin/main`. 

You should find that `git checkout origin/main` has been executed. You can also use `abbr` mode instead of `exec` mode for the second segment, if you don't want it to execute immediately. 

## Top-level keybinds 

Top-level keybinds are hotkeys that you can press in the terminal, to execute a command. They can be set to change automatically as you move between projects. 

Top-level keybinds can be defined in three places: 

1. Globally, in the `global.keybinds` section of the global config. These keybinds are always set, and don't change. 
2. In the `profiles` section of the global config. These keybinds are active only when one of their `conditions` is fulfilled. 
3. In the local config, which may exist at `.blz.yml` in any directory. 

### Global keybinds

In the `all` config, you will see: 

```yml
global:
  # Keybinds are not behind a leader key.
  keybinds:
    - key: "Alt-p"
      zle: push-line # To bind to a Zsh builtin, use the `zle` value.
    - key: "Alt-l"
      command: "ls -lah"
    ...
```

You can try pressing `Alt-l` to run `ls -lah`. You can also bind `zle` builtins here, as seen in the example (they also work in leader-key combos).

### Profile-specific keybinds

In the `all` config, you will see profiles, such as: 

```yml
profiles:
  - name: C++
    conditions:
      # Only activate in any directory underneath "~/projects/cpp/", or if a makefile is present.
      - within: "~/projects/cpp/"
      - glob: "makefile"
    keybinds:
      - key: "Alt-b"
        command: "make -j`nproc`"
  ...
```

Try creating a directory with `mkdir tmp && touch tmp/makefile && cd tmp` and press `Alt-b`. It should run the `make` command from the profile. If you `cd` to a Rust project, `Alt-b` should then run `cargo build`.

> **Tip**: Profiles are evaluated when you `cd`. You can use `cd .` if you need to re-evaluate the profile conditions. 

#### Local configs

In any directory, you can run `blz -l` to create/edit a local config. This can inherit a profile from the global config, for example: 

```yml
# .blz.yml
inherits:
  - Rust
```

It can also define its own keybinds:

```yml
# .blz.yml
keybinds:
  - key: "Ctrl-g"
    command: "!!^M"   # Run the last command.
  - key: "Alt-C"
    command: "cargo check"
```

After creating a local config, use `cd .` to refresh the keybindings.

## Next steps

Now you can update your global config. If you want to regenerate from a template, run `rm ~/.config/blaze-keys/.blz.yml` and then `blz -g`, perhaps choosing the `small`/`minimal` config. 

I would recommend that you change the leader-key trigger keys to something optimal for you. 

