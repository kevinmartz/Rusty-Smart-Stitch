use std::path::PathBuf;
use std::fs;
use std::env;
use std::io;
use serde::{Deserialize, Serialize};
use semver::Version;
use reqwest;
use anyhow::{Result, Context, bail};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "linux")]
use std::os::unix::fs::PermissionsExt;


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
        let current_version = Version::parse(CURRENT_VERSION)
            .context("Failed to parse current version")?;
        
        let executable_path = env::current_exe()
            .context("Failed to get current executable path")?;
            
        let temp_dir = env::temp_dir().join("rusty_smart_stitch_update");
        
        Ok(Self {
            current_version,
            executable_path,
            temp_dir,
        })
    }

    // Asks GitHub if there's a new version available if there is it returns the info about the release if not it returns none. 
    // im new to this github api stuff so i dont know if this is the best way to do it but it works.
    pub async fn check_for_updates(&self) -> Result<Option<ReleaseInfo>> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            GITHUB_OWNER, GITHUB_REPO
        );

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

        // Get rid of the 'v' in front of version numbers
        let release_version = Version::parse(
            release_info.tag_name.trim_start_matches('v')
        ).context("Failed to parse release version")?;

        // checks if the new version is actually newer
        if release_version > self.current_version {
            Ok(Some(release_info))
        } else {
            Ok(None)
        }
    }

    // Downloads new version
    pub async fn download_update(&self, release: &ReleaseInfo) -> Result<PathBuf> {
        fs::create_dir_all(&self.temp_dir)
            .context("Failed to create temporary directory")?;

        let asset = self.get_platform_asset(release)
            .context("No compatible release asset found")?;

        let client = reqwest::Client::new();
        let response = client
            .get(&asset.browser_download_url)
            .send()
            .await
            .context("Failed to download update")?;

        let new_binary_path = self.temp_dir.join(
            self.executable_path
                .file_name()
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Invalid executable name"))?
        );

        let mut file = fs::File::create(&new_binary_path)
            .context("Failed to create temporary file")?;
        
        let content = response.bytes().await
            .context("Failed to get response bytes")?;
            
        io::copy(&mut content.as_ref(), &mut file)
            .context("Failed to write update file")?;

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
        let platform_suffix = if cfg!(target_os = "windows") {
            "windows.exe"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            return None;
        };

        release.assets.iter()
            .find(|asset| asset.name.contains(platform_suffix))
            .cloned()
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

        fs::write(&batch_script, script_content)
            .context("Failed to create update script")?;

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

        fs::write(&shell_script, script_content)
            .context("Failed to create update script")?;

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
