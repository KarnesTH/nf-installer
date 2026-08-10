use std::path::Path;

/// Derives the name to install under from the given path.
pub fn name_from(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let name = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or("could not derive a font name from the path")?;

    Ok(name.to_string())
}

/// Checks that the path exists and points to something we can install from.
pub fn validate(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        return Err(format!("{} does not exist", path.display()).into());
    }

    Ok(())
}
