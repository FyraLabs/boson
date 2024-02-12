use color_eyre::Result;
use std::{
    path::{Path, PathBuf},
    process::ExitStatus,
};
// File names for the Steamworks libraries
static STEAM_RUNTIME_FILES: &[&str] = &["libsdkencryptedappticket.so", "libsteam_api.so"];
static STEAMWORKS_SDK_URL: &str = "https://github.com/calendulish/Overlays/raw/master/dev-util/steamworks-sdk/files/steamworks_sdk_158.zip";
pub struct SteamRuntime;

impl SteamRuntime {
    pub fn install() -> Result<()> {
        let path = runtimes_path().join("steamworks");
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }

        for (i, file) in STEAM_RUNTIME_FILES.iter().enumerate() {
            let file_path = path.join(file);
            if !file_path.exists() {
                // Download the runtime from the internet

                match i {
                    0 => {
                        // extract the sdkencryptedappticket
                        todo!();
                    }
                    1 => {
                        // extract the steam_api
                        todo!();
                    }
                    _ => {
                        unreachable!();
                    }
                }
            }
        }
        // Download the runtime from the internet
        todo!();
        Ok(())
    }
}

pub trait Runtime {
    fn run(
        &self,
        path: &Path,       // Path to the game data
        pwd: &Path,        // Working directory
        args: Vec<String>, // Additional arguments to pass to the game
    ) -> Result<ExitStatus>;

    fn get_version(&self) -> Result<String>;
}

/// Path for runtime data
/// Should be relative to the executable and not the current working directory
pub fn runtimes_path() -> PathBuf {
    // get path of the current executable
    let mut path = std::env::current_exe().unwrap();

    // remove the filename
    path.pop();

    // join runtimes
    path.join("runtimes")
}

pub struct ElectronRuntime {
    pub version: String,
}

impl ElectronRuntime {
    // Folder where the runtime should be located
    pub fn path(&self) -> PathBuf {
        runtimes_path().join("electron").join(self.version.as_str())
    }

    pub fn get_exec_path(&self) -> PathBuf {
        self.path().join("electron")
    }

    // Check if the runtime is installed
    pub fn check_path(&self) -> bool {
        self.path().exists() && self.get_exec_path().exists()
    }

    pub fn new<S: Into<String>>(version: S) -> Self {
        Self {
            version: version.into(),
        }
    }

    pub fn install(&self) -> Result<()> {
        let path = self.path();
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }
        // Download the runtime from the internet
        todo!();
        Ok(())
    }
}

impl Runtime for ElectronRuntime {
    fn run(&self, path: &Path, pwd: &Path, args: Vec<String>) -> Result<ExitStatus> {
        let mut cmd_args = vec![path.to_str().unwrap()];
        cmd_args.extend(args.iter().map(|s| s.as_str()));
        let mut cmd = std::process::Command::new(&self.get_exec_path());
        cmd.current_dir(pwd);
        cmd.args(cmd_args);
        run_with_flags(&mut cmd)
    }

    fn get_version(&self) -> Result<String> {
        Ok(self.version.clone())
    }
}

fn run_with_flags(cmd: &mut std::process::Command) -> Result<ExitStatus> {
    cmd.env_remove("LD_PRELOAD"); // remove LD_PRELOAD
    let status = cmd.status()?;
    Ok(status)
}
/// Basic implementation for users who want to use their own Electron wrapper/binary
pub struct CustomRuntime(String);

impl CustomRuntime {
    pub fn new<S: Into<String>>(s: S) -> Self {
        Self(s.into())
    }
}

impl Runtime for CustomRuntime {
    fn run(&self, path: &Path, pwd: &Path, args: Vec<String>) -> Result<ExitStatus> {
        let mut cmd_args = vec![path.to_str().unwrap()];
        cmd_args.extend(args.iter().map(|s| s.as_str()));
        let mut cmd = std::process::Command::new(&self.0);
        cmd.current_dir(pwd);
        cmd.args(cmd_args);
        run_with_flags(&mut cmd)
    }

    fn get_version(&self) -> Result<String> {
        Ok("Custom Runtime".to_string())
    }
}
