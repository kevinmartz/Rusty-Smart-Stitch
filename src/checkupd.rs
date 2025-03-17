use anyhow::{bail, Context, Result};
use reqwest;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io;
#[cfg(target_os = "linux")]
use std::os::unix::fs::PermissionsExt;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::path::PathBuf;

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_OWNER: &str = "kevinmartz";
const GITHUB_REPO: &str = "Rusty-Smart-Stitch";

#[derive(Serialize, Deserialize, Clone)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub body: String,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

pub struct Updater {
    current_version: Version,
    executable_path: PathBuf,
    temp_dir: PathBuf,
}

impl Updater {
    pub fn new() -> Result<Self> {
        let current_version =
            Version::parse(CURRENT_VERSION).context("Failed to parse current version")?;

        let executable_path =
            env::current_exe().context("Failed to get current executable path")?;

        let temp_dir = env::temp_dir().join("rusty_smart_stitch_update");

        Ok(Self {
            current_version,
            executable_path,
            temp_dir,
        })
    }

    pub async fn test_github_api_access(&self) -> Result<bool> {
        let client = reqwest::Client::new();
        let url = "https://api.github.com/rate_limit";
        
        println!("Testing GitHub API access: {}", url);
        
        match client
            .get(url)
            .header("User-Agent", "rusty-smart-stitch-updater")
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    println!("GitHub API access successful");
                    Ok(true)
                } else {
                    println!("GitHub API access failed: {}", response.status());
                    Ok(false)
                }
            },
            Err(e) => {
                println!("GitHub API access error: {}", e);
                Ok(false)
            }
        }
    }

    // Asks GitHub if there's a new version available if there is it returns the info about the release if not it returns none.
    // im new to this github api stuff so i dont know if this is the best way to do it but it works.
    pub async fn check_for_updates(&self) -> Result<Option<ReleaseInfo>> {
        // First test GitHub API access
        if !self.test_github_api_access().await? {
            bail!("Cannot access GitHub API. Please check your internet connection.");
        }
        
        let client = reqwest::Client::new();
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            GITHUB_OWNER, GITHUB_REPO
        );

        println!("Checking for updates from: {}", url);

        let response = client
            .get(&url)
            .header("User-Agent", "rusty-smart-stitch-updater")
            .send()
            .await
            .context("Failed to fetch release information")?;

        if !response.status().is_success() {
            bail!("Failed to get release info: {}", response.status());
        }

        let release_info: ReleaseInfo = response
            .json()
            .await
            .context("Failed to parse release information")?;

        println!("Found release: {} with {} assets", release_info.tag_name, release_info.assets.len());
        for asset in &release_info.assets {
            println!("Asset: {} - URL: {}", asset.name, asset.browser_download_url);
        }

        // Get rid of the 'v' in front of version numbers
        let release_version = Version::parse(release_info.tag_name.trim_start_matches('v'))
            .context("Failed to parse release version")?;

        // checks if the new version is actually newer
        if release_version > self.current_version {
            println!("New version available: {} (current: {})", release_version, self.current_version);
            Ok(Some(release_info))
        } else {
            println!("Current version is up to date: {}", self.current_version);
            Ok(None)
        }
    }

    // Downloads new version
    pub async fn download_update(&self, release: &ReleaseInfo) -> Result<PathBuf> {
        fs::create_dir_all(&self.temp_dir).context("Failed to create temporary directory")?;

        let asset = match self.get_platform_asset(release) {
            Some(asset) => {
                println!("Found compatible asset: {} - URL: {}", asset.name, asset.browser_download_url);
                asset
            },
            None => {
                println!("Available assets:");
                for asset in &release.assets {
                    println!("- {}", asset.name);
                }
                bail!("No compatible release asset found for this platform");
            }
        };
            
        println!("Downloading asset: {} from URL: {}", asset.name, asset.browser_download_url);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .context("Failed to build HTTP client")?;
            
        let response = match client
            .get(&asset.browser_download_url)
            .header("User-Agent", "rusty-smart-stitch-updater")
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                println!("Download error: {}", e);
                if e.is_timeout() {
                    bail!("Download timed out. Please check your internet connection and try again.");
                } else if e.is_connect() {
                    bail!("Connection error. Please check your internet connection and try again.");
                } else {
                    bail!("Failed to download update: {}", e);
                }
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            println!("HTTP error: {} - {}", status, error_text);
            bail!("Failed to download update: HTTP status {} - {}", status, error_text);
        }

        let new_binary_path = self.temp_dir.join(
            self.executable_path
                .file_name()
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Invalid executable name"))?,
        );

        println!("Saving downloaded file to: {}", new_binary_path.display());
        let mut file =
            fs::File::create(&new_binary_path).context("Failed to create temporary file")?;

        let content = match response.bytes().await {
            Ok(bytes) => {
                println!("Downloaded {} bytes", bytes.len());
                bytes
            },
            Err(e) => {
                println!("Failed to get response bytes: {}", e);
                bail!("Failed to get response bytes: {}", e);
            }
        };

        match io::copy(&mut content.as_ref(), &mut file) {
            Ok(bytes) => println!("Wrote {} bytes to file", bytes),
            Err(e) => {
                println!("Failed to write update file: {}", e);
                bail!("Failed to write update file: {}", e);
            }
        }

        #[cfg(target_family = "unix")]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&new_binary_path, fs::Permissions::from_mode(0o755))
                .context("Failed to set executable permissions")?;
        }

        Ok(new_binary_path)
    }

    // which file to download based on OS
    fn get_platform_asset(&self, release: &ReleaseInfo) -> Option<ReleaseAsset> {
        if cfg!(target_os = "windows") {
            // Try multiple possible Windows asset naming patterns
            let patterns = ["windows.exe", "win.exe", "win64.exe", "win32.exe", "windows-x64.exe", "windows-x86.exe", ".exe"];
            
            for pattern in patterns {
                if let Some(asset) = release.assets.iter().find(|asset| asset.name.to_lowercase().contains(pattern)) {
                    return Some(asset.clone());
                }
            }
            
            // If no specific pattern matches, try to find any .exe file
            if let Some(asset) = release.assets.iter().find(|asset| asset.name.to_lowercase().ends_with(".exe")) {
                return Some(asset.clone());
            }
            
            return None;
        } else if cfg!(target_os = "linux") {
            let patterns = ["linux", "x86_64-unknown-linux-gnu"];
            
            for pattern in patterns {
                if let Some(asset) = release.assets.iter().find(|asset| asset.name.to_lowercase().contains(pattern)) {
                    return Some(asset.clone());
                }
            }
            
            return None;
        } else if cfg!(target_os = "macos") {
            let patterns = ["macos", "darwin", "mac", "x86_64-apple-darwin"];
            
            for pattern in patterns {
                if let Some(asset) = release.assets.iter().find(|asset| asset.name.to_lowercase().contains(pattern)) {
                    return Some(asset.clone());
                }
            }
            
            return None;
        } else {
            return None;
        }
    }

    // Installs that new version based on OS
    pub async fn apply_update(&self, new_binary_path: PathBuf) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            self.apply_update_windows(new_binary_path)
        }

        #[cfg(target_os = "linux")]
        {
            self.apply_update_linux(new_binary_path)
        }

        #[cfg(target_os = "macos")]
        {
            self.apply_update_macos(new_binary_path)
        }
    }

    // Windows update
    #[cfg(target_os = "windows")]
    fn apply_update_windows(&self, new_binary_path: PathBuf) -> Result<()> {
        use std::process::Command;

        // makes a batch script to do the dirty work
        let batch_script = self.temp_dir.join("update.bat");
        let script_content = format!(
            "@echo off\n\
             timeout /t 1 /nobreak >nul\n\
             copy /Y \"{}\" \"{}\"\n\
             start /b \"\" \"{}\"\n\
             del \"%~f0\"\n",
            new_binary_path.display(),
            self.executable_path.display(),
            self.executable_path.display()
        );

        fs::write(&batch_script, script_content).context("Failed to create update script")?;

        // runs the script without showing a window
        Command::new("cmd")
            .args(["/C", batch_script.to_str().unwrap()])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW flag
            .spawn()
            .context("Failed to execute update script")?;

        std::process::exit(0);
    }

    // Linux update
    // this is just a copy and run script there is no linux version and will never be from me!
    // well... i did make a linux version so rejoyce!
    // if you want to make a linux version you can do it yourself! go ahead! be my guest!
    // but nevertheless here is the script for linux! if anyone wants to make a linux version!
    #[cfg(target_os = "linux")]
    fn apply_update_linux(&self, new_binary_path: PathBuf) -> Result<()> {
        use std::process::Command;

        // makes a shell script to do the dirty work
        let shell_script = self.temp_dir.join("update.sh");
        let script_content = format!(
            "#!/bin/bash\n\
             sleep 1\n\
             cp -f \"{}\" \"{}\"\n\
             \"{}\" &\n\
             rm \"$0\"\n",
            new_binary_path.display(),
            self.executable_path.display(),
            self.executable_path.display()
        );

        fs::write(&shell_script, script_content).context("Failed to create update script")?;

        fs::set_permissions(&shell_script, fs::Permissions::from_mode(0o755))
            .context("Failed to set script permissions")?;

        Command::new("sh")
            .arg(&shell_script)
            .spawn()
            .context("Failed to execute update script")?;

        std::process::exit(0);
    }

    // Mac version, like linux i will not make a mac version
    // but here is the script for mac! if anyone wants to make a mac version!
    // the script is the same as the linux one! so you can use that!
    #[cfg(target_os = "macos")]
    fn apply_update_macos(&self, new_binary_path: PathBuf) -> Result<()> {
        self.apply_update_linux(new_binary_path)
    }
}
