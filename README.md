# ⚡ blaze-keys

***Blazing fast Zsh commands with customizable leader-key combos and project-specific keybinds.***

## Demo 

[![demo](https://asciinema.org/a/6lKTjS45GDbXvirE89OSa5aDT.svg)](https://asciinema.org/a/6lKTjS45GDbXvirE89OSa5aDT?autoplay=1)

> **Note**: All the keybinds shown are defined in the `blz` config, and fully customizable.

## Introduction

With `blaze-keys` you can:

- Easily assign hotkeys to execute commands in Zsh, like setting `Alt+g` to run `git status`.
  - Define different hotkeys for different projects generically, which **automatically reassign** as you move between projects.
- Define **leader key** combos which can:
  - Execute commands with minimal key presses, and without leaving the home row - useful for commands like `git status` or `cargo build --release`.
  - Expand into the current line to allow you to add arguments - useful for commands like `git commit -am` or `git checkout -b`.
  - Run zsh builtins, like the useful `push-line`.
- Easily customise hotkeys and leader combos to your preferences.  
  - Set multiple leader keys which activate different combos.
  - Explore your leader key combos with the TUI, which is only ever one hotkey away.

For example, maybe you want to be able to use `Shift+Alt+B` to run `cargo build` in a Rust project, but you want the same shortcut to run `make` in a C project. `blaze-keys` allows you to easily configure this, without needing to run any commands when you `cd` between projects.

> Tip: `blaze-keys` also works in Vim mode; call `bindkey -v` before activating `blaze-keys` in your `~/.zshrc`.

### Future work

- [ ] Improve macOS support (please see the [issue](/../../issues/1)). 
  - It should work fine on macOS (it has been tested briefly), but I'm in no position to guarantee this - especially for different terminal emulators. 
- [ ] Support other shells, such as `fish` (please upvote the [issue](/../../issues/3) if you're interested).

## Try it out

You can try out `blaze-keys` without installing it, by using the Docker image:

```bash
docker run --cap-drop=ALL --security-opt=no-new-privileges -it enhancedprimate/blaze-keys:latest
# Or with podman:
podman run --cap-drop=ALL --security-opt=no-new-privileges -it docker.io/enhancedprimate/blaze-keys:latest
```

> ⚠️ **Warning**: Depending on your terminal emulator and platform, some or all of the hotkeys in the tutorial might not work when running via Docker/podman. 

## Configuration

`blaze-keys` uses two types of configuration:

- The `global` configuration:
  - Contains keybindings which are always applied.
  - Defines profiles which contain sets of keybindings for different types of projects.
    - Profiles can define `conditions` which are used to determine whether they should be applied when you `cd` into a project.
  - Defines leader-key combos.
- `local` configurations (optional):
  - Contain keybindings which are applied only when the `local` configuration is in your current working directory.
  - Can `inherit` a profile from the global config.

## Quick start guide

After completing the setup steps below, you can follow the brief [tutorial](./docs/tutorial.md).

### Download 'blz'

`blz` is a single executable file in a tarball, which you can download from the [release page](../../releases). Extract `blz` from the tarball and make sure it's on your `PATH`.

You can alternatively install with `cargo` (Rust `1.88` or newer):

```bash
cargo install --locked --git https://github.com/enhanced-primate/blaze-keys.git
```

### Update .zshrc

Add a line to your `.zshrc` file and source it:

```bash
echo 'source <(blz --zsh-hook)' >> ~/.zshrc
source ~/.zshrc
```

> ⚠️ **macOS users**: You will need to configure your terminal emulator to use the `Option` key as `Alt`. Please see the [macOS terminal setup guide](./docs/macos_terminal_setup.md) for more information.

### Configure global config

Run `blz -g` to edit the global config (creating from template if not present). The repo includes an example which demonstrates many of the available features: [global.all.yml](./example-configs/templates/global.all.yml). I would suggest that you use the `all` config when prompted, then follow the [tutorial](./docs/tutorial.md) to familiarise yourself with the usage; then you can modify the config as you wish.  

> **Warning**: After adding a new leader key to the global config, you need to `source ~/.zshrc` for the new keybinds to take effect.

### Configure local config

> **Note**: Local configs are optional, and I would typically recommend creating a *profile* in the global config instead.

You can use `blz -l` to edit (creating if not present) a local config in any directory, which will be stored as `.blz.yml`. It will be applied only when you `cd` to that directory.

```yml
# Optional: You can inherit a profile from the global config. 
# This isn't necessary if one of the profile's 'conditions' evaluates to true.
inherits:
  - Rust

keybinds:
  - key: "Ctrl-l"
    command: "ls -la"
  - key: "C-g"
    # '^M' can be used as the `Enter` key. This is useful here because `blz` applies only one `^M`, 
    # which in this case expands `!!` to the last executed command; 
    # another 'Enter' is required to execute the expanded last command.
    command: "!!^M"   
  - key: "F10"
    command: "cargo run"

```

### FAQs

#### If I `cd` into a child directory, will my keybindings be unset?

Every time you `cd`, `blz` will emit the appropriate keybindings based on the local config, if present, and any profiles in the global config, if applicable. It does not explicitly unset any keybindings when a local config or profile becomes non-applicable, so your keybindings will remain set until a conflicting keybind is applied.

#### Why use the leader-key functionality of `blaze-keys` instead of shell aliases?

- You can 'overload' aliases by putting them behind different leader keys, to avoid clashes e.g. where you would like `cc` to run `cargo check` or `conan create`.
- You can execute commands without needing to move your hand to reach the Enter key (`exec` mode). In my experience, this reduces strain on my wrist significantly.
- You can use an alias which expands into the current command line (`abbr` mode). Unlike an alias, the full command will appear in your history.
- You can create postfix aliases, e.g. if you often work with a Docker image `localhost/my-private-image`, you can create an alias with `blaze-keys` which can be used after typing the initial command, like `docker save -o out.tar <trigger blaze-keys to insert 'localhost/my-private-image'>`.
- The `blaze-keys` TUI makes it easier to find an alias which you've forgotten, rather than searching for it in the `alias` output.
- Subjectively, using `blaze-keys` just feels like you have a more direct interface with the terminal. It feels smoother and more fluid, partly because the Enter key becomes more obsolete.

#### How do I diagnose problems?

If your top-level keybindings are not working as expected, you can see what keybindings are being emitted by running `blz -v`:

```bash
$ blz -v
Ctrl-P  ----->  'push-line' (zle builtin)
Alt-l   ----->  'ls -lah'
F2      ----->  'git log'
Alt-B   ----->  'cargo build'
Alt-X   ----->  'cargo run'
...
```

> **Note**: This is directory-dependent and will include all keybindings from profiles active in your current working directory, or local configs.

You can run `blz -B` to see the raw `bindkey` commands which are being executed, in Zsh syntax. You can then manually try to run these `bindkey` commands to see if they work. 

If your TUI is not appearing when triggering a leader key, check for a `.panic.blz` file in the current working directory.

#### What are the security risks?

It's possible for another process to inject into the command which is completed with `blz`, if specifically engineered to do so. However, this is no greater risk than that of a program overwriting your `~/.zshrc` to reassign some aliases to malicious commands.

As a precaution, `blz` will not run as root unless an environment variable (`BLZ_ALLOW_ROOT`) is explicitly set. 


