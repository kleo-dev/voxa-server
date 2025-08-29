use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let root = PathBuf::new();
    let config = voxa_server::ServerConfig::default();
    config.start(&root)
}
