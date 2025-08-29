use std::path::PathBuf;

use voxa_server::{ServerConfig, vfs};

fn main() -> anyhow::Result<()> {
    let root = PathBuf::new();
    let config: ServerConfig = vfs::read_config(&root.join("config.json"))?;

    config.start(&root)
}
