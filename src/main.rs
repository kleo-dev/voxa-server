use std::path::PathBuf;

use voxa_server::{ServerConfig, vfs};

fn main() -> voxa_server::Result<()> {
    let root = PathBuf::from("./");
    let config: ServerConfig = vfs::read_config(&root.join("config.json"))?;
    let mut server = config.build(&root);
    server.run()?;
    Ok(())
}
