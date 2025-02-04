use std::path::{Path, PathBuf};
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

/// Executes a command in the specified working directory.
fn run_command_in_dir(working_dir: &Path, program: &str, args: &[&str]) {
    println!("Executing: {} {:?}", program, args);
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", program])
            .args(args)
            .current_dir(working_dir)
            .spawn()
            .expect("Failed to execute command");
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(program)
            .args(args)
            .current_dir(working_dir)
            .spawn()
            .expect("Failed to execute command");
    }
}

/// Launches the build process and returns a receiver for progress updates.
/// Progress is parsed from output lines matching the pattern "[current/total]".
pub fn create_build_command(
    engine_location: &PathBuf,
    project_name: &str,
    platform: &str,
    optimization_type: &str,
    uproject_location: &PathBuf,
) -> Receiver<f32> {
    let (tx, rx) = mpsc::channel::<f32>();

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

    // Regex to match lines like "[1/2743]".
    let progress_regex = Regex::new(r"\[([0-9]+)/([0-9]+)\]").unwrap();

    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line_result in reader.lines() {
            if let Ok(line) = line_result {
                println!("Build output: {}", line);
                if let Some(caps) = progress_regex.captures(&line) {
                    if let (Some(curr_match), Some(total_match)) = (caps.get(1), caps.get(2)) {
                        if let (Ok(current), Ok(total)) =
                            (curr_match.as_str().parse::<f32>(), total_match.as_str().parse::<f32>())
                        {
                            if total > 0.0 {
                                let progress = current / total;
                                let _ = tx.send(progress);
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
/// Progress is parsed from output lines matching the pattern "[current/total]".
pub fn create_package_command(
    engine_location: &PathBuf,
    platform: &str,
    optimization_type: &str,
    uproject_location: &PathBuf,
) -> Receiver<f32> {
    let (tx, rx) = mpsc::channel::<f32>();

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

    // Regex to match lines like "[1/2743]".
    let progress_regex = Regex::new(r"\[([0-9]+)/([0-9]+)\]").unwrap();

    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line_result in reader.lines() {
            if let Ok(line) = line_result {
                println!("Package output: {}", line);
                if let Some(caps) = progress_regex.captures(&line) {
                    if let (Some(curr_match), Some(total_match)) = (caps.get(1), caps.get(2)) {
                        if let (Ok(current), Ok(total)) =
                            (curr_match.as_str().parse::<f32>(), total_match.as_str().parse::<f32>())
                        {
                            if total > 0.0 {
                                let progress = current / total;
                                let _ = tx.send(progress);
                            }
                        }
                    }
                }
            }
        }
    });

    rx
}
