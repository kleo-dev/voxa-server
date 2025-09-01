use std::path::Path;

use libloading::{Library, Symbol};

use crate::{logger, plugin::DynPlugin, vfs};

logger! {
    const LOGGER "Loader"
}

pub fn load_plugin(path: &Path) -> anyhow::Result<DynPlugin> {
    unsafe {
        LOGGER.info(format!("Loading plugin: {:?}", path));
        let lib = Library::new(path)?;
        let func: Symbol<extern "C" fn() -> DynPlugin> = lib.get(b"load_plugin").unwrap();
        Ok(func())
    }
}

pub fn load_plugins(arr: &mut Vec<DynPlugin>, path: &Path) -> crate::Result<()> {
    LOGGER.info("Loading plugins");
    vfs::dir(path)?;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("dylib") {
                match load_plugin(&path) {
                    Ok(plugin) => {
                        arr.push(plugin);
                    }
                    Err(e) => LOGGER.error(format!("Failed to load plugin {:?}: {}", path, e)),
                }
            }
        }
    } else {
        LOGGER.error(format!("{:?} is not a directory", path));
    }

    Ok(())
}
