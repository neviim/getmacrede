use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;
use crate::models::Device;

const STORAGE_FILE: &str = "devices.json";

pub fn load_devices() -> io::Result<Vec<Device>> {
    if !Path::new(STORAGE_FILE).exists() {
        return Ok(Vec::new());
    }
    let file = File::open(STORAGE_FILE)?;
    let reader = BufReader::new(file);
    let devices = serde_json::from_reader(reader)?;
    Ok(devices)
}

pub fn save_devices(devices: &[Device]) -> io::Result<()> {
    let file = File::create(STORAGE_FILE)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, devices)?;
    Ok(())
}
