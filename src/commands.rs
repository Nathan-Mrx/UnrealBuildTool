use std::path::{PathBuf};
use std::process::{Command, Stdio};
use std::io::{BufReader, BufRead};
use std::sync::mpsc::{self, Receiver};
use regex::Regex;

#[cfg(target_os = "windows")]
const BUILD_SCRIPT: &str = "Build.bat";
#[cfg(target_os = "windows")]
const UAT_SCRIPT: &str = "RunUAT.bat";
#[cfg(target_os = "macos")]
const BUILD_SCRIPT: &str = "Mac/Build.sh";
#[cfg(target_os = "macos")]
const UAT_SCRIPT: &str = "RunUAT.sh";

/// Represents an update from the build/package process.
pub enum ProgressUpdate {
    /// A numeric progress update (value between 0.0 and 1.0)
    Progress(f32),
    /// A stage message update (e.g., "Build started", "Cooking...")
    Stage(String),
    /// The process is finished with a final message.
    Finished(String),
}

/// Launches the build process and returns a receiver for progress updates.
/// Progress is parsed from lines matching the pattern "[current/total]".
pub fn create_build_command(
    engine_location: &PathBuf,
    project_name: &str,
    platform: &str,
    optimization_type: &str,
    uproject_location: &PathBuf,
) -> Receiver<ProgressUpdate> {
    let (tx, rx) = mpsc::channel::<ProgressUpdate>();

    let engine_path = engine_location.parent().unwrap().to_string_lossy();
    let build_bat = format!("{}\\Engine\\Build\\BatchFiles\\{}", engine_path, BUILD_SCRIPT);

    let args = [
        project_name,
        platform,
        optimization_type,
        &uproject_location.to_string_lossy(),
        "-waitmutex",
    ];

    let working_dir = uproject_location.parent().unwrap();

    println!("Build command: {} {:?}", build_bat, args);

    let mut child = Command::new("cmd")
        .args(&["/C", &build_bat])
        .args(&args)
        .current_dir(working_dir)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute build command");

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let progress_regex = Regex::new(r"\[([0-9]+)/([0-9]+)\]").unwrap();

    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line_result in reader.lines() {
            if let Ok(line) = line_result {
                println!("Build output: {}", line);
                if line.contains("BUILD SUCCESSFUL") {
                    let _ = tx.send(ProgressUpdate::Finished("Build finished".to_owned()));
                } else if let Some(caps) = progress_regex.captures(&line) {
                    if let (Some(curr_match), Some(total_match)) = (caps.get(1), caps.get(2)) {
                        if let (Ok(current), Ok(total)) =
                            (curr_match.as_str().parse::<f32>(), total_match.as_str().parse::<f32>())
                        {
                            if total > 0.0 {
                                let progress = current / total;
                                let _ = tx.send(ProgressUpdate::Progress(progress));
                            }
                        }
                    }
                }
            }
        }
    });

    rx
}

/// Launches the package process and returns a receiver for progress updates.
pub fn create_package_command(
    engine_location: &PathBuf,
    platform: &str,
    optimization_type: &str,
    uproject_location: &PathBuf,
) -> Receiver<ProgressUpdate> {
    let (tx, rx) = mpsc::channel::<ProgressUpdate>();

    let engine_path = engine_location.parent().unwrap().to_string_lossy().to_string();
    let uat_bat = format!("{}\\Engine\\Build\\BatchFiles\\{}", engine_path, UAT_SCRIPT);
    let staging_directory = format!(
        "{}\\Builds",
        uproject_location.parent().unwrap().to_string_lossy()
    );

    let args = [
        "BuildCookRun",
        &format!("-project={}", uproject_location.to_string_lossy()),
        "-noP4",
        &format!("-platform={}", platform),
        &format!("-clientconfig={}", optimization_type),
        &format!("-serverconfig={}", optimization_type),
        "-nocompileeditor",
        "-cook",
        "-allmaps",
        "-build",
        "-CookCultures=en",
        "-unversionedcookedcontent",
        "-stage",
        "-package",
        &format!("-stagingdirectory={}", staging_directory),
    ];

    println!("Package command: {} {:?}", uat_bat, args);

    let working_dir = uproject_location.parent().unwrap();

    let mut child = Command::new("cmd")
        .args(&["/C", &uat_bat])
        .args(&args)
        .current_dir(working_dir)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute package command");

    let stdout = child.stdout.take().expect("Failed to capture stdout");

    let percentage_regex = Regex::new(r"(\d+)%").unwrap();

    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line_result in reader.lines() {
            if let Ok(line) = line_result {
                println!("Package output: {}", line);
                if line.contains("********** BUILD COMMAND STARTED **********") {
                    let _ = tx.send(ProgressUpdate::Stage("Build started".into()));
                } else if line.contains("********** BUILD COMMAND COMPLETED **********") {
                    let _ = tx.send(ProgressUpdate::Stage("Build completed".into()));
                } else if line.contains("********** COOK COMMAND STARTED **********") {
                    let _ = tx.send(ProgressUpdate::Stage("Cooking...".into()));
                } else if line.contains("********** COOK COMMAND COMPLETED **********") {
                    let _ = tx.send(ProgressUpdate::Stage("Cook completed".into()));
                } else if line.contains("********** STAGE COMMAND STARTED **********") {
                    let _ = tx.send(ProgressUpdate::Stage("Staging...".into()));
                } else if line.contains("********** PACKAGE COMMAND STARTED **********") {
                    let _ = tx.send(ProgressUpdate::Stage("Packaging...".into()));
                } else if line.contains("********** PACKAGE COMMAND COMPLETED **********") {
                    let _ = tx.send(ProgressUpdate::Stage("Package completed".into()));
                } else if line.contains("BUILD SUCCESSFUL") {
                    // Open the staging directory in the file explorer.
                    if cfg!(target_os = "windows") {
                        let _ = Command::new("explorer").arg(&staging_directory).spawn();
                    } else if cfg!(target_os = "macos") {
                        let _ = Command::new("open").arg(&staging_directory).spawn();
                    }
                    let _ = tx.send(ProgressUpdate::Finished("Package finished".into()));
                } else if let Some(caps) = percentage_regex.captures(&line) {
                    if let Some(num_str) = caps.get(1) {
                        if let Ok(percent) = num_str.as_str().parse::<f32>() {
                            let progress = percent / 100.0;
                            let _ = tx.send(ProgressUpdate::Progress(progress));
                        }
                    }
                }
            }
        }
    });

    rx
}