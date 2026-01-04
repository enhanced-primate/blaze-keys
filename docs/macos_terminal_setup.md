## macOS - configuring terminal emulator

On macOS, the `Option` key will not be mapped to the escape sequence needed to trigger keybinds using `Alt`. This needs to be configured based on the terminal emulator. 

> **Note**: Please post any problems and resolutions in the [issue](/../../issues/1) so that we can improve macOS support. Thank you! 

For the normal terminal in macOS:
1. Open settings/preferences. 
2. Select Profiles. 
3. Go to the 'Keyboard' tab. 
4. Below the list of key bindings, there is a checkbox "Use Option as Meta key". Ensure it is checked. 

For iTerm2 (untested!):

1. Open Settings -> Profiles -> Keys. 
2. Look for the General section. 
3. Find the settings for Left Option key and Right Option key.
4. Set them to `Esc+`.

For VS Code integrated terminal (untested!):

1. Open Settings.
2. Search for `terminal.integrated.macOptionIsMeta` and ensure it is checked.
