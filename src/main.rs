use std::path::PathBuf;

use voxa_server::{ServerConfig, vfs};

fn main() -> voxa_server::Result<()> {
    let root = PathBuf::from("");
    let config: ServerConfig = vfs::read_config(&root.join("config.json"))?;
    config.build(&root).run()?;
    Ok(())
}
