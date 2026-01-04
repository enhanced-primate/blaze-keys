use anyhow::Result;
use std::path::PathBuf;

use blaze_keys::CONFIG_FILE_NAME;

pub fn swap_config(name: &str, config_dir: &str) -> Result<()> {
    let config_dir_path = PathBuf::from(config_dir);
    let standard_name = config_dir_path.join(CONFIG_FILE_NAME);
    let swapped_name = config_dir_path.join(format!("{name}.blz.yml"));

    if !swapped_name.exists() {
        if !standard_name.exists() {
            println!("No config to swap out.");
        } else {
            std::fs::rename(&standard_name, &swapped_name)?;
            println!("Swapped out config to {}", swapped_name.to_str().unwrap());
        }
    } else {
        if standard_name.exists() {
            let mut i = 0;
            loop {
                let backup_name = config_dir_path.join(format!("backup.{i}.blz.yml"));
                if !backup_name.exists() {
                    std::fs::rename(&standard_name, &backup_name)?;
                    println!(
                        "Backed up existing config to {}",
                        backup_name.to_str().unwrap()
                    );
                    break;
                }
                i += 1;
            }
        }
        std::fs::rename(&swapped_name, &standard_name)?;
        println!("Swapped in config from {}", swapped_name.to_str().unwrap());
    }

    Ok(())
}
