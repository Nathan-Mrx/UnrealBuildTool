use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_os = "windows")]
const BUILD_SCRIPT: &str = "Build.bat";
#[cfg(target_os = "windows")]
const UAT_SCRIPT: &str = "RunUAT.bat";
#[cfg(target_os = "macos")]
const BUILD_SCRIPT: &str = "Mac/Build.sh";
#[cfg(target_os = "macos")]
const UAT_SCRIPT: &str = "RunUAT.sh";

/// Runs a command with the given program and arguments in the specified working directory.
fn run_command_in_dir(working_dir: &Path, program: &str, args: &[&str]) {
    println!("Executing: {} {:?}", program, args);
    if cfg!(target_os = "windows") {
        // For Windows, we use cmd /C so that the command is run in a shell.
        Command::new("cmd")
            .args(&["/C", program])
            .args(args)
            .current_dir(working_dir)
            .spawn()
            .expect("Failed to execute command");
    } else {
        // For Unix-like systems, we use sh -c.
        Command::new("sh")
            .arg("-c")
            .arg(program)
            .args(args)
            .current_dir(working_dir)
            .spawn()
            .expect("Failed to execute command");
    }
}

pub fn create_build_command(
    engine_location: &PathBuf,
    project_name: &str,
    platform: &str,
    optimization_type: &str,
    uproject_location: &PathBuf,
) {
    let engine_path = engine_location
        .parent()
        .unwrap()
        .to_string_lossy()
        .to_string();

    // Full path to the build batch file.
    let build_bat = format!("{}\\Engine\\Build\\BatchFiles\\{}", engine_path, BUILD_SCRIPT);

    // Build the list of arguments.
    let args = [
        project_name,
        platform,
        optimization_type,
        &uproject_location.to_string_lossy(),
        "-waitmutex",
    ];

    println!("Build command: {} {:?}", build_bat, args);

    // Use the directory containing the .uproject as the working directory.
    let working_dir = uproject_location.parent().unwrap();
    run_command_in_dir(working_dir, &build_bat, &args);
}

pub fn create_package_command(
    engine_location: &PathBuf,
    platform: &str,
    optimization_type: &str,
    uproject_location: &PathBuf,
) {
    let engine_path = engine_location
        .parent()
        .unwrap()
        .to_string_lossy()
        .to_string();

    // Full path to the UAT (Unreal Automation Tool) script.
    let uat_bat = format!("{}\\Engine\\Build\\BatchFiles\\{}", engine_path, UAT_SCRIPT);

    // Prepare the staging directory argument.
    let staging_directory = format!(
        "{}\\Builds",
        uproject_location.parent().unwrap().to_string_lossy()
    );

    // Build the list of arguments.
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

    // Use the directory containing the .uproject as the working directory.
    let working_dir = uproject_location.parent().unwrap();
    run_command_in_dir(working_dir, &uat_bat, &args);
}
