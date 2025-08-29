use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

pub fn dir(path: &Path) -> crate::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn read(path: &Path, default_content: &str) -> crate::Result<String> {
    if !path.exists() {
        dir(path.parent().ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidFilename,
            "File doesn't have a parent assigned, example: `config/config.json`",
        ))?)?;
        fs::write(path, default_content)?;
        return Ok(default_content.to_string());
    }

    Ok(fs::read_to_string(path)?)
}

pub fn read_bytes<'a>(path: &Path, default_content: Vec<u8>) -> crate::Result<Vec<u8>> {
    if !path.exists() {
        dir(path.parent().ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidFilename,
            "File doesn't have a parent assigned, example: `config/config.json`",
        ))?)?;
        fs::write(path, &default_content)?;
        return Ok(default_content);
    }

    Ok(fs::read(path)?)
}

pub fn read_config<T: Default + Serialize + for<'de> Deserialize<'de>>(
    path: &Path,
) -> crate::Result<T> {
    if !path.exists() {
        dir(path.parent().ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidFilename,
            "File doesn't have a parent assigned, example: `config/config.json`",
        ))?)?;
        let default = T::default();
        fs::write(path, &serde_json::to_string_pretty(&default)?)?;
        return Ok(default);
    }

    let read = fs::read_to_string(path)?;
    Ok(serde_json::from_str::<T>(&read)?)
}

pub fn write(path: &Path, content: &str) -> crate::Result<()> {
    dir(path.parent().ok_or(std::io::Error::new(
        std::io::ErrorKind::InvalidFilename,
        "File doesn't have a parent assigned, example: `config/config.json`",
    ))?)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn write_bytes(path: &Path, content: &[u8]) -> crate::Result<()> {
    dir(path.parent().ok_or(std::io::Error::new(
        std::io::ErrorKind::InvalidFilename,
        "File doesn't have a parent assigned, example: `config/config.json`",
    ))?)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn write_config<T: Serialize>(path: &Path, content: &T) -> crate::Result<()> {
    dir(path.parent().ok_or(std::io::Error::new(
        std::io::ErrorKind::InvalidFilename,
        "File doesn't have a parent assigned, example: `config/config.json`",
    ))?)?;
    fs::write(path, &serde_json::to_string_pretty(content)?)?;
    Ok(())
}
