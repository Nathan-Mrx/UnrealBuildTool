use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Project {
    pub name: String,
    pub location: PathBuf,
    pub engine_version: String,
    pub plugins: Vec<String>,
}

impl Project {
    /// Creates a new Project from the given .uproject file.
    pub fn new(location: PathBuf) -> Self {
        let name = location.file_stem().unwrap().to_string_lossy().to_string();
        let engine_version = Self::get_engine_version(&location);
        let plugins = Self::get_plugins(&location);
        Self {
            name,
            location,
            engine_version,
            plugins,
        }
    }

    fn get_engine_version(location: &PathBuf) -> String {
        let uproject_content = fs::read_to_string(location)
            .expect("Unable to read uproject file");
        let engine_association: serde_json::Value =
            serde_json::from_str(&uproject_content)
                .expect("Unable to parse uproject file");
        if let Some(engine_association) = engine_association.get("EngineAssociation") {
            let engine_association_str = engine_association.as_str().unwrap();
            if engine_association_str.starts_with('{') && engine_association_str.ends_with('}') {
                "From Source".to_string()
            } else {
                engine_association_str.to_string()
            }
        } else {
            "Unknown".to_string()
        }
    }

    fn get_plugins(location: &PathBuf) -> Vec<String> {
        let uproject_content = fs::read_to_string(location)
            .expect("Unable to read uproject file");
        let uproject: serde_json::Value =
            serde_json::from_str(&uproject_content)
                .expect("Unable to parse uproject file");
        if let Some(plugins) = uproject.get("Plugins") {
            plugins
                .as_array()
                .unwrap()
                .iter()
                .map(|plugin| {
                    plugin.get("Name")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string()
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}

pub fn save_project_locations(projects: &[Project]) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(projects)?;
    fs::write("projects.json", json)?;
    println!("projects.json file updated");
    Ok(())
}

pub fn load_project_locations() -> Result<Vec<Project>, Box<dyn std::error::Error>> {
    if !Path::new("projects.json").exists() {
        return Ok(Vec::new());
    }
    let json = fs::read_to_string("projects.json")?;
    let projects: Vec<Project> = serde_json::from_str(&json)?;
    Ok(projects)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Engine {
    pub location: PathBuf,
}

pub fn save_engine_location(location: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine { location };
    let json = serde_json::to_string_pretty(&engine)?;
    fs::write("engine.json", json)?;
    println!("engine.json file updated");
    Ok(())
}

pub fn load_engine_location() -> Result<Option<Engine>, Box<dyn std::error::Error>> {
    if !Path::new("engine.json").exists() {
        return Ok(None);
    }
    let json = fs::read_to_string("engine.json")?;
    let engine: Engine = serde_json::from_str(&json)?;
    Ok(Some(engine))
}
