## Test plan

This is 100% human-generated. 

Follow the test plan for both `zsh` and `nushell`.

1. Delete `~/.config/blaze-keys`, remove snippets in shell configs.
2. Follow instructions for shell integration.
3. Create global config based on template. 

### Top-level keybinds 

  - Test keybinds that change based on profiles. 
  - Test keybinds that change based on local configs.
  - Test in directories which do not contain local config.

### Leader keys

- Test leader keys in exec mode.
- Test leader keys in abbr mode. 
- Change leader key trigger and check that we are warned correctly. 
- Check that the updated trigger takes effect. 
- Test composing commands from multiple invocations. 

### Config 

- Test behaviour when global config contains an invalid leader key trigger and combo.
- Test behaviour when local config contains invalid key definition.
- Test behaviour when global config is invalid structure. 
- Test behaviour when local config is invalid structure. 

