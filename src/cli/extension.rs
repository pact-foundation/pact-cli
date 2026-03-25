use std::{
    collections::HashMap,
    env, fs,
    io::Write,
    path::Path,
    process::{Command as Cmd, ExitStatus},
};

use clap::{value_parser, Arg, ArgMatches, Command};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionConfig {
    pub name: String,
    pub version: String,
    pub binary_path: String,
    pub extension_type: ExtensionType,
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtensionType {
    Drift,
    PactflowAi,
    PactRubyStandalone,
    External,
}

pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
}

impl PlatformInfo {
    pub fn detect() -> Self {
        let os = match env::consts::OS {
            "macos" => "darwin",
            "windows" => "windows",
            other => other,
        }
        .to_string();

        let arch = match env::consts::ARCH {
            "aarch64" => "aarch64",
            "x86_64" => "x86_64",
            other => other,
        }
        .to_string();

        Self { os, arch }
    }

    pub fn is_supported(&self) -> bool {
        let supported_platforms = [
            ("darwin", "aarch64"),
            ("darwin", "x86_64"),
            ("windows", "aarch64"),
            ("windows", "x86_64"),
            ("linux", "aarch64"),
            ("linux", "x86_64"),
        ];

        supported_platforms.contains(&(self.os.as_str(), self.arch.as_str()))
    }

    pub fn get_pactflow_ai_url(&self) -> String {
        let target = match (self.os.as_str(), self.arch.as_str()) {
            ("darwin", "aarch64") => "aarch64-apple-darwin",
            ("darwin", "x86_64") => "x86_64-apple-darwin",
            ("windows", "aarch64") => "aarch64-pc-windows-msvc",
            ("windows", "x86_64") => "x86_64-pc-windows-msvc",
            ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
            ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
            _ => {
                eprintln!(
                    "Unsupported OS and architecture combination: os={}, arch={}",
                    self.os, self.arch
                );
                std::process::exit(1);
            }
        };

        format!("https://download.pactflow.io/ai/dist/{}/latest", target)
    }

    pub fn get_pactflow_ai_download_url(&self, version: &str) -> String {
        let target = match (self.os.as_str(), self.arch.as_str()) {
            ("darwin", "aarch64") => "aarch64-apple-darwin",
            ("darwin", "x86_64") => "x86_64-apple-darwin",
            ("windows", "aarch64") => "aarch64-pc-windows-msvc",
            ("windows", "x86_64") => "x86_64-pc-windows-msvc",
            ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
            ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
            _ => {
                eprintln!(
                    "Unsupported OS and architecture combination: os={}, arch={}",
                    self.os, self.arch
                );
                std::process::exit(1);
            }
        };

        format!(
            "https://download.pactflow.io/ai/dist/{}/{}/pactflow-ai",
            target, version
        )
    }
    pub fn get_drift_url(&self) -> String {
        format!("https://download.pactflow.io/drift/version.txt")
    }
    pub fn get_drift_download_url(&self, version: &str) -> String {
        let target = match (self.os.as_str(), self.arch.as_str()) {
            ("darwin", "aarch64") => "macos-aarch64",
            ("darwin", "x86_64") => "macos-x86_64",
            ("windows", "aarch64") => "windows-x86_64", // no arm64 support, rely on prism
            ("windows", "x86_64") => "windows-x86_64",
            ("linux", "aarch64") => "linux-aarch64",
            ("linux", "x86_64") => "linux-x86_64",
            _ => {
                eprintln!(
                    "Unsupported OS and architecture combination: os={}, arch={}",
                    self.os, self.arch
                );
                std::process::exit(1);
            }
        };

        format!(
            "https://download.pactflow.io/drift/{}/{}.tgz",
            version, target
        )
    }

    pub fn get_ruby_standalone_target(&self) -> String {
        match (self.os.as_str(), self.arch.as_str()) {
            ("darwin", "aarch64") => "osx-arm64",
            ("darwin", "x86_64") => "osx-x86_64",
            ("windows", "aarch64") => "windows-aarch64", // Windows arm64 not available
            ("windows", "x86_64") => "windows-x86_64",
            ("linux", "aarch64") => "linux-arm64",
            ("linux", "x86_64") => "linux-x86_64",
            _ => {
                eprintln!(
                    "Unsupported OS and architecture combination: os={}, arch={}",
                    self.os, self.arch
                );
                std::process::exit(1);
            }
        }
        .to_string()
    }

    pub fn get_executable_extension(&self) -> &str {
        if self.os == "windows" {
            ".exe"
        } else {
            ""
        }
    }

    pub fn get_archive_extension(&self) -> &str {
        if self.os == "windows" {
            "zip"
        } else {
            "tar.gz"
        }
    }
}

pub struct ExtensionManager {
    pub extensions_home: String,
    pub drift_home: String,
    pub platform: PlatformInfo,
}

impl ExtensionManager {
    pub fn new() -> Self {
        let home_dir = home::home_dir().unwrap_or_default();
        let extensions_home = env::var("PACT_CLI_EXTENSIONS_HOME")
            .unwrap_or_else(|_| home_dir.join(".pact/extensions").display().to_string());
        let drift_home = home_dir.join(".drift").display().to_string();

        Self {
            extensions_home,
            drift_home,
            platform: PlatformInfo::detect(),
        }
    }

    pub fn ensure_extensions_dir(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.extensions_home)
    }

    pub fn get_extension_config_path(&self) -> String {
        format!("{}/config.json", self.extensions_home)
    }

    pub fn load_config(&self) -> HashMap<String, ExtensionConfig> {
        let config_path = self.get_extension_config_path();
        if let Ok(content) = fs::read_to_string(&config_path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        }
    }

    pub fn save_config(&self, config: &HashMap<String, ExtensionConfig>) -> std::io::Result<()> {
        let config_path = self.get_extension_config_path();
        let json = serde_json::to_string_pretty(config)?;
        self.ensure_extensions_dir()?;
        fs::write(config_path, json)
    }

    pub fn list_extensions(&self) -> HashMap<String, ExtensionConfig> {
        let mut config = self.load_config();

        // Add built-in extensions if not present
        let builtin_extensions = [
            ("drift", ExtensionType::Drift),
            ("pactflow-ai", ExtensionType::PactflowAi),
            ("pact-broker-legacy", ExtensionType::PactRubyStandalone),
            ("pactflow-legacy", ExtensionType::PactRubyStandalone),
            ("message-legacy", ExtensionType::PactRubyStandalone),
            ("mock-legacy", ExtensionType::PactRubyStandalone),
            ("verifier-legacy", ExtensionType::PactRubyStandalone),
            ("stub-legacy", ExtensionType::PactRubyStandalone),
        ];

        for (name, ext_type) in builtin_extensions {
            if !config.contains_key(name) {
                let binary_path = format!(
                    "{}/bin/{}{}",
                    self.extensions_home,
                    name,
                    self.platform.get_executable_extension()
                );
                let installed = Path::new(&binary_path).exists();

                config.insert(
                    name.to_string(),
                    ExtensionConfig {
                        name: name.to_string(),
                        version: "latest".to_string(),
                        binary_path,
                        extension_type: ext_type,
                        installed,
                    },
                );
            }
        }

        config
    }

    pub async fn install_pactflow_ai(
        &self,
        version: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !self.platform.is_supported() {
            return Err(format!(
                "Unsupported platform: {}-{}",
                self.platform.os, self.platform.arch
            )
            .into());
        }

        self.ensure_extensions_dir()?;

        let version = if let Some(v) = version {
            v.to_string()
        } else {
            self.get_latest_pactflow_ai_version().await?
        };

        let url = self
            .platform
            .get_pactflow_ai_download_url(&version.replace("+", "%2b"));

        println!("🚀 Downloading pactflow-ai from {}", url);

        let response = reqwest::get(&url).await?;
        if !response.status().is_success() {
            return Err(
                format!("Failed to download pactflow-ai: HTTP {}", response.status()).into(),
            );
        }

        let body = response.bytes().await?;
        let bin_dir = format!("{}/bin", self.extensions_home);
        fs::create_dir_all(&bin_dir)?;

        let binary_path = format!(
            "{}/pactflow-ai{}",
            bin_dir,
            self.platform.get_executable_extension()
        );
        let mut file = fs::File::create(&binary_path)?;
        file.write_all(&body)?;

        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = file.metadata()?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&binary_path, perms)?;
        }

        // Update config
        let mut config = self.load_config();
        config.insert(
            "pactflow-ai".to_string(),
            ExtensionConfig {
                name: "pactflow-ai".to_string(),
                version: version.to_string(),
                binary_path,
                extension_type: ExtensionType::PactflowAi,
                installed: true,
            },
        );
        self.save_config(&config)?;

        println!("✅ Successfully installed pactflow-ai");
        Ok(())
    }
    pub async fn install_drift(
        &self,
        version: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !self.platform.is_supported() {
            return Err(format!(
                "Unsupported platform: {}-{}",
                self.platform.os, self.platform.arch
            )
            .into());
        }

        // Install to ~/.drift
        fs::create_dir_all(&self.drift_home)?;

        let version = if let Some(v) = version {
            v.to_string()
        } else {
            self.get_latest_drift_version().await?
        };

        let url = self.platform.get_drift_download_url(&version);

        println!("🚀 Downloading drift from {}", url);

        let response = reqwest::get(&url).await?;
        if !response.status().is_success() {
            return Err(format!("Failed to download drift: HTTP {}", response.status()).into());
        }

        let body = response.bytes().await?;
        let archive_path = format!("{}/drift.tar.gz", self.drift_home);
        let mut file = fs::File::create(&archive_path)?;
        file.write_all(&body)?;
        drop(file);

        // Extract archive to ~/.drift
        println!("🚀 Extracting drift...");
        Self::extract_drift_archive_to(&archive_path, &self.drift_home)?;

        // Clean up archive
        fs::remove_file(&archive_path)?;

        // Update config - preserve existing extensions
        let mut config = self.load_config();
        let drift_bin = format!("{}/drift", self.drift_home);
        config.insert(
            "drift".to_string(),
            ExtensionConfig {
                name: "drift".to_string(),
                version: version.to_string(),
                binary_path: drift_bin.clone(),
                extension_type: ExtensionType::Drift,
                installed: true,
            },
        );
        self.save_config(&config)?;

        println!("✅ Successfully installed drift to ~/.drift");
        Ok(())
    }

    pub async fn install_ruby_legacy(
        &self,
        version: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !self.platform.is_supported() {
            return Err(format!(
                "Unsupported platform: {}-{}",
                self.platform.os, self.platform.arch
            )
            .into());
        }

        self.ensure_extensions_dir()?;

        // Get latest version if not specified
        let version = if let Some(v) = version {
            v.to_string()
        } else {
            self.get_latest_ruby_standalone_version().await?
        };

        let target = self.platform.get_ruby_standalone_target();
        let archive_ext = self.platform.get_archive_extension();
        let url =
            format!(
            "https://github.com/pact-foundation/pact-standalone/releases/download/{}/pact-{}-{}.{}",
            version, version.trim_start_matches('v'), target, archive_ext
        );

        println!("🚀 Downloading pact-legacy from {}", url);

        let response = reqwest::get(&url).await?;
        if !response.status().is_success() {
            return Err(
                format!("Failed to download pact-legacy: HTTP {}", response.status()).into(),
            );
        }

        let body = response.bytes().await?;
        let archive_path = format!("{}/pact-legacy.{}", self.extensions_home, archive_ext);
        let mut file = fs::File::create(&archive_path)?;
        file.write_all(&body)?;
        drop(file);

        // Extract archive
        println!("🚀 Extracting pact-legacy...");
        self.extract_ruby_archive(&archive_path)?;

        // Create symlinks for legacy commands and record installed version
        self.create_legacy_symlinks_with_version(&version)?;

        // Clean up archive
        fs::remove_file(&archive_path)?;

        println!("✅ Successfully installed pact-legacy tools");
        Ok(())
    }

    async fn get_latest_ruby_standalone_version(
        &self,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = "https://api.github.com/repos/pact-foundation/pact-standalone/releases/latest";
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .header("User-Agent", "pact-cli")
            .send()
            .await?;

        let release: serde_json::Value = response.json().await?;
        let tag_name = release["tag_name"]
            .as_str()
            .ok_or("No tag_name found in release")?;

        Ok(tag_name.to_string())
    }

    async fn get_latest_pactflow_ai_version(&self) -> Result<String, Box<dyn std::error::Error>> {
        let url = self.platform.get_pactflow_ai_url();
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("User-Agent", "pact-cli")
            .send()
            .await?;

        let text = response.text().await?;
        // The API returns just the version number like "1.11.4"
        Ok(text.trim().to_string())
    }

    fn get_installed_pactflow_ai_version(&self) -> Result<String, Box<dyn std::error::Error>> {
        let config = self.load_config();
        if let Some(ext_config) = config.get("pactflow-ai") {
            if ext_config.installed && Path::new(&ext_config.binary_path).exists() {
                let output = Cmd::new(&ext_config.binary_path)
                    .arg("--version")
                    .output()?;

                if output.status.success() {
                    let version_output = String::from_utf8_lossy(&output.stdout);
                    // Parse version from output like "pactflow-ai 2.0.10 (ae6a94e1 2026-03-03)"
                    if let Some(version) = version_output.split_whitespace().nth(1) {
                        if let Some(commit) = version_output.split_whitespace().nth(2) {
                            let commit = commit.trim_start_matches('(').trim_end_matches(')');
                            return Ok(format!("{}+{}", version, commit));
                        }
                        return Ok(version.to_string());
                    }
                }
            }
        }
        Ok("unknown".to_string())
    }
    async fn get_latest_drift_version(&self) -> Result<String, Box<dyn std::error::Error>> {
        let url = self.platform.get_drift_url();
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("User-Agent", "pact-cli")
            .send()
            .await?;

        let text: String = response.text().await?;
        // The API returns just the version number like "1.11.4"
        Ok(text.trim().to_string())
    }

    fn get_installed_drift_version(&self) -> Result<String, Box<dyn std::error::Error>> {
        let config = self.load_config();
        if let Some(_ext_config) = config.get("drift") {
            let drift_executable_path = format!(
                "{}/drift{}",
                self.drift_home,
                self.platform.get_executable_extension()
            );

            println!("{drift_executable_path}");
            if Path::new(&drift_executable_path).exists() {
                let output = Cmd::new(drift_executable_path).arg("--version").output()?;

                if output.status.success() {
                    let version_output = String::from_utf8_lossy(&output.stdout);
                    println!("{version_output}");
                    // Parse version from output like "Drift testing tools 2603.0.1-beta"
                    if let Some(version) = version_output.split_whitespace().nth(3) {
                        return Ok(version.to_string());
                    }
                }
            }
        }
        Ok("unknown".to_string())
    }

    fn extract_ruby_archive(&self, archive_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let extract_dir = format!("{}/pact-legacy", self.extensions_home);
        fs::create_dir_all(&extract_dir)?;

        if self.platform.os == "windows" {
            // Use PowerShell for Windows
            let status = Cmd::new("powershell")
                .arg("-Command")
                .arg(format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    archive_path, extract_dir
                ))
                .status()?;

            if !status.success() {
                return Err("Failed to extract Windows archive".into());
            }
        } else {
            // Use tar for Unix systems
            let status = Cmd::new("tar")
                .arg("-xzf")
                .arg(archive_path)
                .arg("-C")
                .arg(&extract_dir)
                .arg("--strip-components=1")
                .status()?;

            if !status.success() {
                return Err("Failed to extract tar archive".into());
            }
        }

        Ok(())
    }
    /// Extracts drift archive to the specified directory (used for ~/.drift install)
    fn extract_drift_archive_to(
        archive_path: &str,
        extract_dir: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(extract_dir)?;
        let status = Cmd::new("tar")
            .arg("-xzf")
            .arg(archive_path)
            .arg("-C")
            .arg(extract_dir)
            .status()?;

        if !status.success() {
            return Err("Failed to extract tar archive".into());
        }

        Ok(())
    }

    fn create_legacy_symlinks_with_version(
        &self,
        version: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let bin_dir = format!("{}/bin", self.extensions_home);
        fs::create_dir_all(&bin_dir)?;

        let ruby_bin_dir = format!("{}/pact-legacy/bin", self.extensions_home);
        let exe_ext = self.platform.get_executable_extension();

        let legacy_mappings = [
            ("pact-broker", "pact-broker-legacy"),
            ("pactflow", "pactflow-legacy"),
            ("pact-message", "message-legacy"),
            ("pact-mock-service", "mock-legacy"),
            ("pact-provider-verifier", "verifier-legacy"),
            ("pact-stub-service", "stub-legacy"),
        ];

        for (source_name, target_name) in legacy_mappings {
            let source_path = format!("{}/{}{}", ruby_bin_dir, source_name, exe_ext);
            let target_path = format!("{}/{}{}", bin_dir, target_name, exe_ext);

            if Path::new(&source_path).exists() {
                #[cfg(unix)]
                {
                    if Path::new(&target_path).exists() {
                        fs::remove_file(&target_path)?;
                    }
                    std::os::unix::fs::symlink(&source_path, &target_path)?;
                }

                #[cfg(windows)]
                {
                    fs::copy(&source_path, &target_path)?;
                }

                println!(
                    "📋 Created legacy mapping: {} -> {}",
                    target_name, source_name
                );
            }
        }

        // Update config for all legacy tools
        let mut config = self.load_config();

        // Add master pact-legacy entry
        let ruby_dir = format!("{}/pact-legacy", self.extensions_home);
        config.insert(
            "pact-legacy".to_string(),
            ExtensionConfig {
                name: "pact-legacy".to_string(),
                version: version.to_string(),
                binary_path: ruby_dir.clone(),
                extension_type: ExtensionType::PactRubyStandalone,
                installed: Path::new(&ruby_dir).exists(),
            },
        );

        for (_, target_name) in legacy_mappings {
            let binary_path = format!("{}/{}{}", bin_dir, target_name, exe_ext);
            let installed = Path::new(&binary_path).exists();

            config.insert(
                target_name.to_string(),
                ExtensionConfig {
                    name: target_name.to_string(),
                    version: version.to_string(),
                    binary_path,
                    extension_type: ExtensionType::PactRubyStandalone,
                    installed,
                },
            );
        }
        self.save_config(&config)?;

        Ok(())
    }

    pub fn run_extension(
        &self,
        extension_name: &str,
        args: &[String],
    ) -> Result<ExitStatus, Box<dyn std::error::Error>> {
        let config = self.load_config();

        if let Some(ext_config) = config.get(extension_name) {
            if !ext_config.installed {
                return Err(format!(
                    "Extension '{}' is not installed. Run 'pact extension install {}' first.",
                    extension_name, extension_name
                )
                .into());
            }

            let binary_path = if extension_name == "drift" {
                format!(
                    "{}/drift{}",
                    self.drift_home,
                    self.platform.get_executable_extension()
                )
            } else {
                ext_config.binary_path.clone()
            };

            let status = Cmd::new(binary_path).args(args).status()?;

            Ok(status)
        } else {
            // Try to find external binary
            let binary_name = format!("pact-{}", extension_name);
            match Cmd::new(&binary_name).args(args).status() {
                Ok(status) => Ok(status),
                Err(_) => Err(format!("Extension '{}' not found. Available extensions can be listed with 'pact extension list'.", extension_name).into()),
            }
        }
    }

    pub fn uninstall_extension(
        &self,
        extension_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut config = self.load_config();
        if extension_name == "pact-legacy" {
            // Special handling for master ruby-standalone extension
            println!("🗑️  Uninstalling pact-legacy and all legacy tools...");

            // Remove all legacy tool symlinks and config entries
            let legacy_tools: Vec<String> = config
                .iter()
                .filter_map(|(name, ext_config)| {
                    if matches!(ext_config.extension_type, ExtensionType::PactRubyStandalone)
                        && name != "pact-legacy"
                    {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
                .collect();

            for tool in &legacy_tools {
                if let Some(tool_config) = config.get(tool) {
                    if Path::new(&tool_config.binary_path).exists() {
                        fs::remove_file(&tool_config.binary_path)?;
                        println!("🗑️  Removed legacy tool: {}", tool);
                    }
                }
                config.remove(tool);
            }

            // Remove the ruby-standalone directory
            let ruby_dir = format!("{}/pact-legacy", self.extensions_home);
            if Path::new(&ruby_dir).exists() {
                fs::remove_dir_all(&ruby_dir)?;
                println!("🗑️  Removed ruby-standalone directory");
            }

            // Remove master config entry
            config.remove("pact-legacy");
            self.save_config(&config)?;

            println!("✅ Successfully uninstalled pact-legacy and all legacy tools");
        } else if let Some(_ext_config) = config.get(extension_name) {
            println!("🗑️  Uninstalling extension: {}", extension_name);

            // Remove drift binary if uninstalling drift
            if extension_name == "drift" {
                fs::remove_dir_all(&self.drift_home)?;
                println!("🗑️  Removed drift directory: {}", self.drift_home);
            }

            config.remove(extension_name);
            self.save_config(&config)?;
            println!("✅ Successfully uninstalled extension: {}", extension_name);
        } else {
            return Err(format!("Extension '{}' is not installed.", extension_name).into());
        }

        Ok(())
    }
}

pub fn add_extension_subcommand() -> Command {
    Command::new("extension")
        .about("Manage Pact CLI extensions")
        .allow_external_subcommands(true)
        .external_subcommand_value_parser(value_parser!(String))
        .subcommand(
            Command::new("list")
                .about("List available and installed extensions")
                .arg(
                    Arg::new("installed")
                        .long("installed")
                        .help("Show only installed extensions")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("install")
                .about("Install an extension")
                .arg(
                    Arg::new("extension")
                        .help("Extension name to install")
                        .required(false)
                        .value_parser(["pactflow-ai", "pact-legacy", "drift"]),
                )
                .arg(
                    Arg::new("all")
                        .long("all")
                        .help("Update all installed extensions")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("version")
                        .long("version")
                        .help("Specific version to install (defaults to latest)")
                        .num_args(1),
                ),
        )
        .subcommand(
            Command::new("update")
                .about("Update extensions")
                .arg(
                    Arg::new("extension")
                        .help("Extension name to update (optional)")
                        .required(false),
                )
                .arg(
                    Arg::new("all")
                        .long("all")
                        .help("Update all installed extensions")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("uninstall")
                .about("Uninstall an extension")
                .arg(
                    Arg::new("extension")
                        .help("Extension name to uninstall")
                        .required(false),
                )
                .arg(
                    Arg::new("all")
                        .long("all")
                        .help("Update all installed extensions")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
}

pub async fn run_extension_command(args: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let manager = ExtensionManager::new();

    match args.subcommand() {
        Some(("list", sub_args)) => {
            let installed_only = sub_args.get_flag("installed");
            let extensions = manager.list_extensions();

            // Fetch latest versions from APIs
            let latest_ruby_version = match manager.get_latest_ruby_standalone_version().await {
                Ok(v) => v,
                Err(_) => "unknown".to_string(),
            };
            let latest_pactflow_ai_version = match manager.get_latest_pactflow_ai_version().await {
                Ok(v) => v,
                Err(_) => "unknown".to_string(),
            };
            let latest_drift_version = match manager.get_latest_drift_version().await {
                Ok(v) => v,
                Err(_) => "unknown".to_string(),
            };

            println!("📦 Available extensions:");

            let mut table = comfy_table::Table::new();
            table
                .set_header(vec!["Name", "Type", "Installed", "Latest", "Status"])
                .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);

            for (name, config) in extensions {
                if installed_only && !config.installed {
                    continue;
                }

                let ext_type = match config.extension_type {
                    ExtensionType::Drift => "Drift",
                    ExtensionType::PactflowAi => "PactFlow AI",
                    ExtensionType::PactRubyStandalone => "Pact Legacy",
                    ExtensionType::External => "External",
                };

                let status = if config.installed {
                    "✅ Installed"
                } else {
                    "❌ Not Installed"
                };
                let installed_version = if config.installed {
                    if matches!(config.extension_type, ExtensionType::PactflowAi) {
                        match manager.get_installed_pactflow_ai_version() {
                            Ok(v) => v,
                            Err(_) => "unknown".to_string(),
                        }
                    } else if matches!(config.extension_type, ExtensionType::Drift) {
                        match manager.get_installed_drift_version() {
                            Ok(v) => v,
                            Err(_) => "unknown".to_string(),
                        }
                    } else {
                        config.version.clone()
                    }
                } else {
                    "-".to_string()
                };

                let latest_version =
                    if matches!(config.extension_type, ExtensionType::PactRubyStandalone) {
                        latest_ruby_version.clone()
                    } else if matches!(config.extension_type, ExtensionType::PactflowAi) {
                        latest_pactflow_ai_version.clone()
                    } else if matches!(config.extension_type, ExtensionType::Drift) {
                        latest_drift_version.clone()
                    } else {
                        "-".to_string()
                    };

                table.add_row(vec![
                    name,
                    ext_type.to_string(),
                    installed_version,
                    latest_version,
                    status.to_string(),
                ]);
            }

            println!("{}", table);
        }
        Some(("install", sub_args)) => {
            let extension = sub_args.get_one::<String>("extension");
            let version = sub_args.get_one::<String>("version").map(|s| s.as_str());
            let all = sub_args.get_flag("all");

            if all {
                println!("🚀 Installing all available extensions...");
                manager.install_pactflow_ai(version).await?;
                manager.install_ruby_legacy(version).await?;
                manager.install_drift(version).await?;
            } else if let Some(ext_name) = extension {
                match ext_name.as_str() {
                    "pactflow-ai" => {
                        manager.install_pactflow_ai(version).await?;
                    }
                    "pact-legacy" => {
                        manager.install_ruby_legacy(version).await?;
                    }
                    "drift" => {
                        manager.install_drift(version).await?;
                    }
                    _ => {
                        return Err(format!("Unknown extension: {}", ext_name).into());
                    }
                }
            } else {
                return Err("Please specify an extension name or use --all flag".into());
            }
        }
        Some(("update", sub_args)) => {
            let all = sub_args.get_flag("all");
            let extension = sub_args.get_one::<String>("extension");

            if all {
                let extensions = manager.list_extensions();
                let installed_extensions: Vec<_> = extensions
                    .iter()
                    .filter(|(_, config)| config.installed)
                    .collect();

                if installed_extensions.is_empty() {
                    println!("⚠️  No extensions are currently installed. Use 'pact extension install' to install extensions first.");
                    return Err("No extensions installed".into());
                }

                for (name, config) in installed_extensions {
                    println!("🔄 Updating {}...", name);
                    match config.extension_type {
                        ExtensionType::PactflowAi => {
                            manager.install_pactflow_ai(None).await?;
                        }
                        ExtensionType::Drift => {
                            manager.install_drift(None).await?;
                        }
                        ExtensionType::PactRubyStandalone => {
                            manager.install_ruby_legacy(None).await?;
                        }
                        ExtensionType::External => {
                            println!("⚠️  Cannot update external extension: {}", name);
                        }
                    }
                }
            } else if let Some(ext_name) = extension {
                let extensions = manager.list_extensions();
                if let Some(config) = extensions.get(ext_name) {
                    if config.installed {
                        println!("🔄 Updating {}...", ext_name);
                        match config.extension_type {
                            ExtensionType::PactflowAi => {
                                manager.install_pactflow_ai(None).await?;
                            }
                            ExtensionType::Drift => {
                                manager.install_drift(None).await?;
                            }
                            ExtensionType::PactRubyStandalone => {
                                manager.install_ruby_legacy(None).await?;
                            }
                            ExtensionType::External => {
                                println!("⚠️  Cannot update external extension: {}", ext_name);
                            }
                        }
                    } else {
                        return Err(format!("Extension '{}' is not installed", ext_name).into());
                    }
                } else {
                    return Err(format!("Extension '{}' not found", ext_name).into());
                }
            } else {
                return Err("Please specify an extension name or use --all flag".into());
            }
        }
        Some(("uninstall", sub_args)) => {
            let extension = sub_args.get_one::<String>("extension");
            let all = sub_args.get_flag("all");

            if all {
                let extensions = manager.list_extensions();
                let mut installed_extensions: Vec<_> = extensions
                    .iter()
                    .filter(|(_, config)| config.installed)
                    .map(|(name, config)| (name.clone(), config.clone()))
                    .collect();

                // For PactRubyStandalone extensions, only keep the master entry
                let mut ruby_found = false;
                installed_extensions.retain(|(name, config)| {
                    if matches!(config.extension_type, ExtensionType::PactRubyStandalone) {
                        if !ruby_found && name == "pact-legacy" {
                            ruby_found = true;
                            true
                        } else {
                            false
                        }
                    } else {
                        true
                    }
                });

                if installed_extensions.is_empty() {
                    println!("⚠️  No extensions are currently installed.");
                    return Ok(());
                }

                println!("🗑️  Uninstalling all extensions...");
                for (ext_name, _) in installed_extensions {
                    manager.uninstall_extension(&ext_name)?;
                }
            } else if let Some(ext_name) = extension {
                manager.uninstall_extension(ext_name)?;
            } else {
                return Err("Please specify an extension name or use --all flag".into());
            }
        }
        Some((external_cmd, _)) => {
            // Handle external subcommands - pass through to extension
            let mut args: Vec<String> = std::env::args().collect();

            // Find the position of "extension" and remove everything before and including it
            if let Some(pos) = args.iter().position(|x| x == "extension") {
                args.drain(0..=pos);
            }

            if !args.is_empty() {
                let extension_name = &args[0];
                let extension_args = &args[1..];

                match manager.run_extension(extension_name, extension_args) {
                    Ok(status) => {
                        if !status.success() {
                            std::process::exit(status.code().unwrap_or(1));
                        }
                    }
                    Err(e) => return Err(e),
                }
            }
        }
        None => {
            println!(
                "⚠️  No subcommand provided. Use 'pact extension --help' for available commands."
            );
        }
    }

    Ok(())
}

pub fn run_external_extension(
    extension_name: &str,
    args: &[String],
) -> Result<ExitStatus, Box<dyn std::error::Error>> {
    let manager = ExtensionManager::new();
    manager.run_extension(extension_name, args)
}

/// Get list of installed pactflow extensions
pub fn get_pactflow_extensions() -> Vec<String> {
    let manager = ExtensionManager::new();
    let extensions = manager.list_extensions();

    extensions
        .iter()
        .filter_map(|(name, config)| {
            if config.installed && name.starts_with("pactflow-") {
                // Return just the extension part (e.g., "ai" from "pactflow-ai")
                Some(name.strip_prefix("pactflow-").unwrap_or(name).to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Check if a command is a pactflow extension
pub fn is_pactflow_extension(command: &str) -> bool {
    let pactflow_extensions = get_pactflow_extensions();
    pactflow_extensions.contains(&command.to_string())
}

/// Run a pactflow extension by mapping command to binary name
pub fn run_pactflow_extension(
    extension_cmd: &str,
    args: &[String],
) -> Result<ExitStatus, Box<dyn std::error::Error>> {
    let binary_name = format!("pactflow-{}", extension_cmd);
    let manager = ExtensionManager::new();
    manager.run_extension(&binary_name, args)
}
