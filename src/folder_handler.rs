use std::path::PathBuf;
use crate::RustySmartStitchApp;
use native_dialog::FileDialog;
use std::path::Path;
use eframe::egui;
use std::collections::VecDeque;

impl RustySmartStitchApp {

    pub fn add_folder_contents(&mut self, folder_path: PathBuf) -> bool {
        println!("Processing folder: {:?}", folder_path);

        self.root_input_path = Some(folder_path.clone());
        self.input_paths.clear();
        
        // Set the root output directory with the selected format
        if let Some(parent) = folder_path.parent() {
            let root_name = folder_path.file_name().unwrap_or_default().to_string_lossy();
            let output_root_name = format!("{}_{}", root_name, self.output_format);
            self.root_output_dir = Some(parent.join(output_root_name));
            self.output_dir = Some(self.root_output_dir.as_ref().unwrap().clone());
            self.manual_output_dir = self.output_dir.as_ref().unwrap().to_string_lossy().to_string();
            println!("Set root output dir to: {:?}", self.root_output_dir);
        }
        
        let mut subfolder_queue = VecDeque::new();

        // First grab all the files from the main folder
        if let Ok(entries) = std::fs::read_dir(&folder_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        let ext = ext.to_string_lossy().to_lowercase();
                        if ["png", "jpg", "jpeg", "webp", "bmp", "psd"].contains(&ext.as_str()) {
                            println!("Adding file: {:?}", path);
                            self.input_paths.push(path);
                        }
                    }
                } else if path.is_dir() {
                    println!("Found subfolder: {:?}", path);
                    subfolder_queue.push_back(path);
                }
            }
        }

        if self.input_paths.is_empty() && !subfolder_queue.is_empty() {
            // Sort it so it's not random
            let mut subfolder_vec: Vec<_> = subfolder_queue.drain(..).collect();
            subfolder_vec.sort();
            
            while let Some(current_folder) = subfolder_vec.first() {
                println!("Checking subfolder: {:?}", current_folder);
                let mut has_files = false;
                let mut inner_subfolders = VecDeque::new();
                
                if let Ok(entries) = std::fs::read_dir(current_folder) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(ext) = path.extension() {
                                let ext = ext.to_string_lossy().to_lowercase();
                                if ["png", "jpg", "jpeg", "webp", "bmp", "psd"].contains(&ext.as_str()) {
                                    println!("Adding file: {:?}", path);
                                    self.input_paths.push(path);
                                    has_files = true;
                                }
                            }
                        } else if path.is_dir() {
                            inner_subfolders.push_back(path);
                        }
                    }
                }

                if has_files {
                    if let Some(root_dir) = &self.root_output_dir {
                        if let Ok(rel_path) = current_folder.strip_prefix(&folder_path) {
                            self.output_dir = Some(root_dir.join(rel_path));
                            println!("Set subfolder output dir to: {:?}", self.output_dir);
                        }
                    }

                    // Queue remaining folders
                    let mut remaining_queue = VecDeque::new();
                    remaining_queue.extend(subfolder_vec[1..].iter().cloned());
                    remaining_queue.extend(inner_subfolders);
                    if !remaining_queue.is_empty() {
                        self.pending_subfolders = Some(remaining_queue);
                    }

                    self.input_paths.sort();
                    return true;
                } else {
                    subfolder_vec.remove(0); // Remove current empty folder
                    subfolder_vec.extend(inner_subfolders);
                    if subfolder_vec.is_empty() {
                        return false;
                    }
                }
            }
            return false;
        } else {
            self.input_paths.sort();
        }

        if !subfolder_queue.is_empty() {
            println!("Storing {} subfolders for later processing", subfolder_queue.len());
            self.pending_subfolders = Some(subfolder_queue);
        } else {
            self.pending_subfolders = None;
        }

        println!("Set output dir to: {:?}", self.output_dir);
        true
    }

    pub fn process_next_subfolder(&mut self) -> bool {
        if let Some(ref mut queue) = self.pending_subfolders {
            if let Some(subfolder) = queue.pop_front() {
                println!("Processing subfolder: {:?}", subfolder);
                
                self.input_paths.clear();
                let mut has_files = false;

                if let Ok(entries) = std::fs::read_dir(&subfolder) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(ext) = path.extension() {
                                let ext = ext.to_string_lossy().to_lowercase();
                                if ["png", "jpg", "jpeg", "webp", "bmp", "psd"].contains(&ext.as_str()) {
                                    println!("Adding file: {:?}", path);
                                    self.input_paths.push(path);
                                    has_files = true;
                                }
                            }
                        } else if path.is_dir() {
                            println!("Found nested folder: {:?}", path);
                            queue.push_back(path);
                        }
                    }
                }
                self.input_paths.sort();

                // Use the root output directory as base and maintain the subfolder structure
                if let (Some(root_dir), Some(root_input)) = (&self.root_output_dir, &self.root_input_path) {
                    if let Ok(rel_path) = subfolder.strip_prefix(root_input) {
                        self.output_dir = Some(root_dir.join(rel_path));
                        println!("Set subfolder output dir to: {:?}", self.output_dir);
                    }
                }

                if has_files {
                    true
                } else if !queue.is_empty() {
                    self.process_next_subfolder()
                } else {
                    self.pending_subfolders = None;
                    false
                }
            } else {
                self.pending_subfolders = None;
                false
            }
        } else {
            false
        }
    }

    pub fn handle_select_folder(&mut self) {
        if let Ok(Some(path)) = FileDialog::new().show_open_single_dir() {
            self.add_folder_contents(path.clone());
            self.success_message.clear();
            self.input_paths.sort();
        }
    }

    pub fn update_output_directory_from_path(&mut self, path: &PathBuf) {
        if let Some(parent) = path.parent() {
            if let Some(grandparent) = parent.parent() {
                let output_dir_name = format!(
                    "{}_{}",
                    parent.file_name().unwrap().to_string_lossy(),
                    self.output_format
                );
                self.output_dir = Some(grandparent.join(output_dir_name));
                self.manual_output_dir = self.output_dir.as_ref().unwrap().to_string_lossy().to_string();
            }
        }
    }

    pub fn handle_output_dir_selection(&mut self) {
        if let Ok(Some(path)) = FileDialog::new().show_open_single_dir() {
            self.output_dir = Some(path.clone());
            self.manual_output_dir = path.to_string_lossy().to_string();
        }
    }

    pub fn handle_select_files(&mut self, supported_extensions: &[&str]) {
        if let Ok(paths) = FileDialog::new()
            .add_filter("Images", supported_extensions)
            .show_open_multiple_file()
        {
            self.input_paths = paths;
            self.success_message.clear();
            self.input_paths.sort();
            
            if let Some(first_path) = self.input_paths.first().cloned() {
                self.update_output_directory_from_path(&first_path);
            }
        }
    }

    pub fn clear_all(&mut self) {
        self.input_paths.clear();
        self.output_dir = None;
        self.manual_output_dir.clear();
        self.error_message.clear();
        self.success_message.clear();
        self.pending_subfolders = None;
    }

    pub fn handle_file_drops(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() && self.input_paths.is_empty() {
                for dropped_file in &i.raw.dropped_files {
                    if let Some(path) = &dropped_file.path {
                        if path.is_dir() {
                            self.add_folder_contents(path.clone());
                        } else {
                            self.handle_dropped_file(path);
                        }
                    }
                }
                self.input_paths.sort();
            }
        });
    }

    pub fn handle_dropped_file(&mut self, path: &PathBuf) {
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if ["png", "jpg", "jpeg", "webp", "bmp", "psd"].contains(&ext.as_str()) {
                self.input_paths.push(path.clone());
                self.update_output_directory(path);
            }
        }
    }

    pub fn update_output_directory(&mut self, path: &PathBuf) {
        if let Some(parent) = path.parent() {
            let output_dir_name = format!(
                "{}_{}", 
                parent.file_name().unwrap().to_string_lossy(),
                self.output_format
            );
            self.output_dir = Some(parent.join(output_dir_name));
            self.manual_output_dir = self.output_dir.as_ref().unwrap().to_string_lossy().to_string();
        }
    }

    pub fn setup_output_directory(&mut self) {
        // If we have a root input path, always create a new root output directory with current format
        if let Some(root_input) = &self.root_input_path {
            if let Some(parent) = root_input.parent() {
                let root_name = root_input.file_name().unwrap_or_default().to_string_lossy();
                let output_root_name = format!("{}_{}", root_name, self.output_format);
                self.root_output_dir = Some(parent.join(output_root_name));
                
                // Update subfolder path if we have input files
                if let Some(first_path) = self.input_paths.first() {
                    if let Ok(rel_path) = first_path.parent().unwrap().strip_prefix(root_input) {
                        self.output_dir = Some(self.root_output_dir.as_ref().unwrap().join(rel_path));
                    } else {
                        self.output_dir = self.root_output_dir.clone();
                    }
                } else {
                    self.output_dir = self.root_output_dir.clone();
                }
                
                if let Some(root_dir) = &self.root_output_dir {
                    self.manual_output_dir = root_dir.to_string_lossy().to_string();
                }
                return;
            }
        }

        // Fallback: If we don't have a root input path but have some input files
        if self.output_dir.is_none() {
            if let Some(first_path) = self.input_paths.first() {
                if let Some(parent) = first_path.parent() {
                    let dir_name = parent.file_name()
                        .unwrap_or_default()
                        .to_string_lossy();
                    let output_dir = parent.with_file_name(format!("{}_{}", dir_name, self.output_format));
                    self.output_dir = Some(output_dir.clone());
                    self.manual_output_dir = output_dir.to_string_lossy().to_string();
                    println!("Created new output directory: {:?}", self.output_dir);
                }
            }
        }
    }
    // Collects image files from output directory for waifu2x
    pub fn collect_image_files(dir: &Path) -> Result<Vec<std::fs::DirEntry>, std::io::Error> {
        Ok(std::fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.to_lowercase())
                    .map(|ext| ["jpg", "jpeg", "png", "bmp"].contains(&ext.as_str()))
                    .unwrap_or(false)
            })
            .collect())
    }
} 