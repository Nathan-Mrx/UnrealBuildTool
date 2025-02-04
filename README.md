# UnrealBuildTool

UnrealBuildTool is a graphical user interface (GUI) application written in Rust using [eframe](https://github.com/emilk/egui/tree/master/eframe) and [egui](https://github.com/emilk/egui). It simplifies the process of building and packaging Unreal Engine projects by allowing you to select an engine solution, choose a project, configure build settings, and launch build/packaging commands—all while providing real-time progress updates.

## Features

- **Engine Selection:** Easily select your Unreal Engine solution file (e.g. `UE5.sln`).
- **Project Management:** Open and manage multiple Unreal project files (`.uproject`).
- **Build Configuration:** Choose between Debug, Development, and Shipping modes.
- **Platform Selection:** Target platforms such as Win64, Linux, Mac, Android, iOS, PS4, PS5, XBoxOne, XBoxSeries, and Switch.
- **Build & Package:** Execute build and package commands with real-time progress updates.
- **Progress Feedback:** Display a progress bar based on the build output (parsed from trace lines like `[1/2743]`).
- **Responsive UI:** Buttons are automatically disabled during a build or packaging process.

## Requirements

- **Rust:** Install the latest stable version from [rust-lang.org](https://www.rust-lang.org/).
- **Cargo:** Comes with Rust installation.
- **Platform:** Designed primarily for Windows (the executable packaging produces a `.exe` file).

## Installation

Clone the repository:

```bash
git clone https://github.com/yourusername/UnrealBuildTool.git
cd UnrealBuildTool
```
## Building the Application
Build the application in debug mode:

```bash
cargo build
```

Build the application in release mode (recommended for distribution):
```bash
cargo build --release
```

The resulting executable will be located in the target/release directory (e.g. `target/release/UnrealBuildTool.exe` on Windows).

## Running the Application
Run the application using Cargo:

```bash
cargo run
```

Or run the standalone executable (after building in release mode):

```bash
target/release/UnrealBuildTool.exe
```

## Usage
1. **Open Engine**:
Click the Open Engine button to select your Unreal Engine solution file (UE5.sln).

2. **Open Project**:
Click the Open Project button to select your Unreal project file (.uproject). Your projects will then be listed for selection.

3. **Select Build Configuration**:
Choose the build mode (Debug, Development, or Shipping) and target platform (e.g., Win64, Linux, Mac, etc.) using the radio buttons.

4. **Build / Package**:
Click the Build button to launch the build process or the Package button to package the project (the Package button is enabled only if the project is built from source). While a process is running, both buttons are disabled.
The progress bar below the buttons will update in real time based on the output trace (e.g. progress is computed from lines like `[1/2743]`).

## Packaging & Distribution
To create a standalone executable for Windows:

1. Build in release mode:
```bash
cargo build --release
```
2. The resulting executable is located in the `target/release` directory (e.g. `UnrealBuildTool.exe`).
3. Distribute the executable along with any necessary configuration files or assets. For a polished installation, consider using an installer tool such as Inno Setup or NSIS.

## Contributing
Contributions, issues, and feature requests are welcome! Please check the issues section or open a pull request.

## License
This project is licensed under the MIT License.

## Contact
Nathan Merieux – nathan.merieux@outlook.fr
Project Link: https://github.com/Nathan-Mrx/UnrealBuildTool


