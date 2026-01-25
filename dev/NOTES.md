# nushell

config file at: ~/.config/nushell/config.nu

## top level keybinds

Will need to register a env_change hook:

$env.config.hooks = {
    env_change: {
        PWD: [{|before, after| '$env.test = (blz porcelain blat)' }]
    }
}

How do I make that update the $env in the current session? It seems like the env changes are scoped to the hook. 

