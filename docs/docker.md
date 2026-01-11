## Trying out blaze-keys with Docker

I created a Docker image as an easy way to test out `blz`, but unfortunately the Docker/podman experience is a bit flaky across different terminals/systems. The keybindings using `Ctrl` are more likely to work than the keybindings using `Alt`, and if on macOS you will need to follow the [instructions](./macos_terminal_setup.md) for your terminal.

Nevertheless, you can try it with the following: 

```bash
docker run --cap-drop=ALL --security-opt=no-new-privileges -it enhancedprimate/blaze-keys:latest
# Or with podman:
podman run --cap-drop=ALL --security-opt=no-new-privileges -it docker.io/enhancedprimate/blaze-keys:latest
```

> ⚠️ **Warning**: Depending on your terminal emulator and platform, some or all of the hotkeys in the tutorial might not work when running via Docker/podman. 


