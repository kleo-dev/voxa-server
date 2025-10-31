use std::path::PathBuf;

use voxa_server::{ServerConfig, utils::vfs};

fn main() -> voxa_server::Result<()> {
    let root = PathBuf::from("");
    let config: ServerConfig = if let Ok(env_config) = std::env::var("VX_CONFIG") {
        ServerConfig::from_str(&env_config)?
    } else {
        vfs::read_config(&root.join("config.json"))?
    };
    config.build(&root).run()?;
    Ok(())
}
