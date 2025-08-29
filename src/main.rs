fn main() -> anyhow::Result<()> {
    let config = voxa_server::ServerConfig::default();
    config.start()
}
