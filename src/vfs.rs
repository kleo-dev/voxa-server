use std::{fs, path::Path};

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
