use eframe::egui;
use rfd::FileDialog;

use crate::storage;
use crate::commands::{create_build_command, create_package_command};

pub struct BuildApp {
    projects: Vec<storage::Project>,
    selected_mode: BuildMode,
    engine_location: Option<storage::Engine>,
    selected_project: Option<usize>,
    selected_platform: Platform,
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
        }
    }
}

impl eframe::App for BuildApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Open Engine button at the top
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

            ui.separator(); // Separator between engine and projects

            // Project radio buttons
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

            ui.separator(); // Separator between projects and build modes

            // Radio buttons for build modes
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.selected_mode, BuildMode::Debug, "Debug");
                ui.radio_value(&mut self.selected_mode, BuildMode::Development, "Development");
                ui.radio_value(&mut self.selected_mode, BuildMode::Shipping, "Shipping");
            });

            ui.separator(); // Separator between build modes and platforms

            // Radio buttons for platforms
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

            // Build and Package buttons at the bottom
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Build").clicked() {
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
                                create_build_command(&engine.location, &project.name, platform, optimization_type, &project.location);
                            } else {
                                eprintln!("No project selected");
                            }
                        } else {
                            eprintln!("No engine location selected");
                        }
                    }

                    let package_button = ui.add_enabled(
                        self.selected_project
                            .map(|index| self.projects[index].engine_version == "From Source")
                            .unwrap_or(false),
                        egui::Button::new("Package"),
                    );

                    if package_button.clicked() {
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
                                create_package_command(&engine.location, platform, optimization_type, &project.location);
                            } else {
                                eprintln!("No project selected");
                            }
                        } else {
                            eprintln!("No engine location selected");
                        }
                    }
                });
            });
        });
    }
}