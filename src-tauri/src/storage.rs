use std::fs;
use std::path::PathBuf;

pub fn data_dir() -> Result<PathBuf, String> {
    let docs = dirs::document_dir().ok_or("无法获取 Documents 目录".to_string())?;
    let dir = docs.join("T-Countdown");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(dir)
}

pub fn data_file_path() -> Result<PathBuf, String> {
    Ok(data_dir()?.join("data.json"))
}

pub fn config_file_path() -> Result<PathBuf, String> {
    Ok(data_dir()?.join("config.json"))
}

pub fn load_data() -> Result<String, String> {
    let path = data_file_path()?;
    if !path.exists() {
        return Ok("[]".to_string());
    }
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

pub fn save_data(json: String) -> Result<(), String> {
    let path = data_file_path()?;
    fs::write(&path, json).map_err(|e| e.to_string())
}
