use std::fs;
use zed_extension_api::{self as zed, LanguageServerId, Result};

struct TableFmtExtension {
    cached_binary_path: Option<String>,
}

impl zed::Extension for TableFmtExtension {
    fn new() -> Self {
        TableFmtExtension {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary_path = self.get_or_download_binary(language_server_id, worktree)?;

        Ok(zed::Command {
            command: binary_path,
            args: vec![],
            env: vec![],
        })
    }
}

impl TableFmtExtension {
    fn get_or_download_binary(
        &mut self,
        _language_server_id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<String> {
        // Check if we have a cached binary that still exists
        if let Some(ref path) = self.cached_binary_path {
            if fs::metadata(path).is_ok() {
                return Ok(path.clone());
            }
        }

        // Determine the platform-specific binary name
        let (platform, arch) = zed::current_platform();

        let (os_name, binary_name) = match platform {
            zed::Os::Mac => ("apple-darwin", "tablefmt-lsp"),
            zed::Os::Linux => ("unknown-linux-gnu", "tablefmt-lsp"),
            zed::Os::Windows => ("pc-windows-msvc", "tablefmt-lsp.exe"),
        };

        let arch_name = match arch {
            zed::Architecture::Aarch64 => "aarch64",
            zed::Architecture::X8664 => "x86_64",
            zed::Architecture::X86 => {
                return Err("x86 (32-bit) is not supported".into());
            }
        };

        let asset_name = format!("tablefmt-lsp-{}-{}.tar.gz", arch_name, os_name);

        // Get the latest release from GitHub
        let release = zed::latest_github_release(
            "nglange/tablefmt",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        // Find the matching asset
        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| format!("No asset found for platform: {}", asset_name))?;

        // Determine download directory based on version
        let version_dir = format!("tablefmt-lsp-{}", release.version);
        let binary_path = format!("{}/{}", version_dir, binary_name);

        // Check if we already have this version
        if fs::metadata(&binary_path).is_ok() {
            self.cached_binary_path = Some(binary_path.clone());
            return Ok(binary_path);
        }

        // Create the version directory
        fs::create_dir_all(&version_dir)
            .map_err(|e| format!("Failed to create directory: {}", e))?;

        // Download and extract the binary
        zed::download_file(
            &asset.download_url,
            &version_dir,
            zed::DownloadedFileType::GzipTar,
        )
        .map_err(|e| format!("Failed to download binary: {}", e))?;

        // Make the binary executable
        zed::make_file_executable(&binary_path)
            .map_err(|e| format!("Failed to make binary executable: {}", e))?;

        // Cache the path for future calls
        self.cached_binary_path = Some(binary_path.clone());

        Ok(binary_path)
    }
}

zed::register_extension!(TableFmtExtension);
