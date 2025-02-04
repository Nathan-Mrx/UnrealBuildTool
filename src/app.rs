use eframe::egui;
use rfd::FileDialog;
use std::sync::mpsc::Receiver;

use crate::storage;
use crate::commands::{create_build_command, create_package_command, ProgressUpdate};

/// Main application state.
pub struct BuildApp {
    projects: Vec<storage::Project>,
    selected_mode: BuildMode,
    engine_location: Option<storage::Engine>,
    selected_project: Option<usize>,
    selected_platform: Platform,
    build_progress: Option<f32>,       // Progress value (0.0 to 1.0)
    progress_message: String,          // Status message to display
    progress_rx: Option<Receiver<ProgressUpdate>>, // Receiver for progress updates
}

#[derive(PartialEq)]
enum BuildMode {
    Debug,
    Development,
    Shipping,
}

#[derive(PartialEq)]
enum Platform {
    Win64,
    Linux,
    Mac,
    Android,
    IOS,
    PS4,
    PS5,
    XBoxOne,
    XBoxSeries,
    Switch,
}

impl Default for BuildApp {
    fn default() -> Self {
        let projects = storage::load_project_locations().unwrap_or_default();
        let engine_location = storage::load_engine_location().unwrap_or_default();
        println!("Loaded projects: {:?}", projects);
        println!("Loaded engine location: {:?}", engine_location);
        Self {
            projects,
            selected_mode: BuildMode::Development,
            engine_location,
            selected_project: None,
            selected_platform: Platform::Win64,
            build_progress: None,
            progress_message: "Idle".to_owned(),
            progress_rx: None,
        }
    }
}

impl eframe::App for BuildApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll for progress updates from the running process.
        {
            if let Some(rx) = self.progress_rx.as_mut() {
                let mut finished = false;
                while let Ok(update) = rx.try_recv() {
                    match update {
                        ProgressUpdate::Progress(p) => {
                            self.build_progress = Some(p);
                            if (p - 1.0).abs() < 0.001 {
                                self.progress_message = "finalizing...".to_owned();
                            } else {
                                self.progress_message = format!("{:.0}% complete", p * 100.0);
                            }
                        }
                        ProgressUpdate::Stage(msg) => {
                            self.progress_message = msg;
                        }
                        ProgressUpdate::Finished(msg) => {
                            self.build_progress = None;
                            self.progress_message = msg;
                            finished = true;
                        }
                    }
                }
                if finished {
                    self.progress_rx = None;
                }
            }
        }

        // The upper part of the UI: Engine, Project, Build Mode, and Platform selections.
        egui::CentralPanel::default().show(ctx, |ui| {
            // Engine Selection
            ui.horizontal(|ui| {
                if ui.button("Open Engine").clicked() {
                    if let Some(file) = FileDialog::new()
                        .add_filter("Solution File", &["sln"])
                        .pick_file()
                    {
                        println!("Selected engine file: {:?}", file);
                        if file.file_name().unwrap() == "UE5.sln" {
                            if let Err(e) = storage::save_engine_location(file.clone()) {
                                eprintln!("Failed to save engine location: {}", e);
                            } else {
                                self.engine_location = Some(storage::Engine { location: file });
                                println!("Engine location saved: {:?}", self.engine_location);
                            }
                        } else {
                            eprintln!("Selected file is not UE5.sln");
                        }
                    }
                }
                if let Some(engine) = &self.engine_location {
                    ui.label(engine.location.to_string_lossy());
                }
            });
            ui.separator();

            // Project Selection
            ui.horizontal_wrapped(|ui| {
                if ui.button("Open Project").clicked() {
                    if let Some(file) = FileDialog::new()
                        .add_filter("Unreal Project", &["uproject"])
                        .pick_file()
                    {
                        println!("Selected project file: {:?}", file);
                        if let Some(existing_index) = self.projects.iter().position(|p| p.location == file) {
                            self.selected_project = Some(existing_index);
                            println!("Project already exists, selected project index: {:?}", self.selected_project);
                        } else {
                            let new_project = storage::Project::new(file);
                            println!("New project added: {:?}", new_project);
                            self.projects.push(new_project.clone());
                            self.selected_project = Some(self.projects.len() - 1);
                            println!("Selected project index: {:?}", self.selected_project);
                            if let Err(e) = storage::save_project_locations(&self.projects) {
                                eprintln!("Failed to save project locations: {}", e);
                            }
                        }
                    }
                }
                for (index, project) in self.projects.iter().enumerate() {
                    let project_info = format!(
                        "{} (Engine: {}, Plugins: {})",
                        project.name,
                        project.engine_version,
                        project.plugins.join(", ")
                    );
                    ui.radio_value(&mut self.selected_project, Some(index), project_info);
                }
            });
            ui.separator();

            // Build Mode Selection
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.selected_mode, BuildMode::Debug, "Debug");
                ui.radio_value(&mut self.selected_mode, BuildMode::Development, "Development");
                ui.radio_value(&mut self.selected_mode, BuildMode::Shipping, "Shipping");
            });
            ui.separator();

            // Platform Selection
            ui.horizontal_wrapped(|ui| {
                ui.radio_value(&mut self.selected_platform, Platform::Win64, "Win64");
                ui.radio_value(&mut self.selected_platform, Platform::Linux, "Linux");
                ui.radio_value(&mut self.selected_platform, Platform::Mac, "Mac");
                ui.radio_value(&mut self.selected_platform, Platform::Android, "Android");
                ui.radio_value(&mut self.selected_platform, Platform::IOS, "iOS");
                ui.radio_value(&mut self.selected_platform, Platform::PS4, "PS4");
                ui.radio_value(&mut self.selected_platform, Platform::PS5, "PS5");
                ui.radio_value(&mut self.selected_platform, Platform::XBoxOne, "XBoxOne");
                ui.radio_value(&mut self.selected_platform, Platform::XBoxSeries, "XBoxSeries");
                ui.radio_value(&mut self.selected_platform, Platform::Switch, "Switch");
            });
        });

        // Compute flags for the bottom panel.
        let running = self.build_progress.is_some();
        let package_condition = self.selected_project
            .map(|index| self.projects[index].engine_version == "From Source")
            .unwrap_or(false);

        // Bottom panel for Build & Package buttons and the progress bar.
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.add_enabled(!running, egui::Button::new("Build")).clicked() {
                    if let Some(engine) = &self.engine_location {
                        if let Some(selected_project_index) = self.selected_project {
                            let project = &self.projects[selected_project_index];
                            let platform = match self.selected_platform {
                                Platform::Win64 => "Win64",
                                Platform::Linux => "Linux",
                                Platform::Mac => "Mac",
                                Platform::Android => "Android",
                                Platform::IOS => "iOS",
                                Platform::PS4 => "PS4",
                                Platform::PS5 => "PS5",
                                Platform::XBoxOne => "XBoxOne",
                                Platform::XBoxSeries => "XBoxSeries",
                                Platform::Switch => "Switch",
                            };
                            let optimization_type = match self.selected_mode {
                                BuildMode::Debug => "Debug",
                                BuildMode::Development => "Development",
                                BuildMode::Shipping => "Shipping",
                            };
                            let rx = create_build_command(
                                &engine.location,
                                &project.name,
                                platform,
                                optimization_type,
                                &project.location,
                            );
                            self.progress_rx = Some(rx);
                            self.build_progress = Some(0.0);
                            self.progress_message = "Build started...".to_owned();
                        } else {
                            eprintln!("No project selected");
                        }
                    } else {
                        eprintln!("No engine location selected");
                    }
                }

                if ui.add_enabled(!running && package_condition, egui::Button::new("Package")).clicked() {
                    if let Some(engine) = &self.engine_location {
                        if let Some(selected_project_index) = self.selected_project {
                            let project = &self.projects[selected_project_index];
                            let platform = match self.selected_platform {
                                Platform::Win64 => "Win64",
                                Platform::Linux => "Linux",
                                Platform::Mac => "Mac",
                                Platform::Android => "Android",
                                Platform::IOS => "iOS",
                                Platform::PS4 => "PS4",
                                Platform::PS5 => "PS5",
                                Platform::XBoxOne => "XBoxOne",
                                Platform::XBoxSeries => "XBoxSeries",
                                Platform::Switch => "Switch",
                            };
                            let optimization_type = match self.selected_mode {
                                BuildMode::Debug => "Debug",
                                BuildMode::Development => "Development",
                                BuildMode::Shipping => "Shipping",
                            };
                            let rx = create_package_command(
                                &engine.location,
                                platform,
                                optimization_type,
                                &project.location,
                            );
                            self.progress_rx = Some(rx);
                            self.build_progress = Some(0.0);
                            self.progress_message = "Packaging started...".to_owned();
                        } else {
                            eprintln!("No project selected");
                        }
                    } else {
                        eprintln!("No engine location selected");
                    }
                }
            });
            if let Some(progress) = self.build_progress {
                ui.add(egui::ProgressBar::new(progress).text(&self.progress_message));
            } else {
                ui.label(&self.progress_message);
            }
        });
        ctx.request_repaint();
    }
}
