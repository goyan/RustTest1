use std::path::Path;
use std::fs;
use std::time::SystemTime;
use serde::Serialize;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[derive(Serialize)]
pub struct FileDescription {
    /// Short type description (e.g. "ZIP Archive", "Rust Source Code")
    pub file_type: String,
    /// Contextual info lines
    pub details: Vec<String>,
    /// Project context if detected
    pub project: Option<String>,
    /// Safety tip for deletion
    pub safety_tip: Option<String>,
}

#[tauri::command]
pub async fn describe_file(name: String, is_dir: bool, path: String) -> Result<FileDescription, String> {
    let path_clone = path.clone();
    let name_clone = name.clone();
    tauri::async_runtime::spawn_blocking(move || {
        build_description(&name_clone, is_dir, &path_clone)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))
}

fn build_description(name: &str, is_dir: bool, path: &str) -> FileDescription {
    let file_path = Path::new(path);
    let name_lower = name.to_lowercase();

    let mut details = Vec::new();

    // --- Metadata ---
    if let Ok(meta) = fs::metadata(file_path) {
        if !is_dir {
            details.push(format!("Size: {}", format_size_detailed(meta.len())));
        }
        if let Ok(modified) = meta.modified() {
            details.push(format!("Modified: {}", format_time(modified)));
        }
        if let Ok(created) = meta.created() {
            details.push(format!("Created: {}", format_time(created)));
        }
    }

    // --- Folder-specific info ---
    if is_dir {
        // Count contents
        if let Ok(entries) = fs::read_dir(file_path) {
            let mut files = 0u32;
            let mut dirs = 0u32;
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() { dirs += 1; } else { files += 1; }
                }
                if files + dirs > 5000 { break; }
            }
            details.push(format!("Contains: {} files, {} folders", files, dirs));
        }

        // Check for README inside
        let readme_path = file_path.join("README.md");
        if readme_path.exists() {
            if let Ok(content) = fs::read_to_string(&readme_path) {
                let first_line = content.lines()
                    .find(|l| !l.trim().is_empty() && !l.starts_with('#'))
                    .or_else(|| content.lines().find(|l| l.starts_with('#')))
                    .unwrap_or("")
                    .trim()
                    .trim_start_matches('#')
                    .trim();
                if !first_line.is_empty() && first_line.len() < 200 {
                    details.push(format!("README: {}", first_line));
                }
            }
        }

        let file_type = get_folder_type(name);
        let safety_tip = get_folder_safety_tip(name);
        let project = detect_project_context(file_path);

        return FileDescription { file_type, details, project, safety_tip };
    }

    // --- File-specific info ---
    let ext = Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    // For .exe and .dll: try PE version info, then fall back to known-exe database
    if ext == "exe" || ext == "dll" {
        let pe_info = get_pe_version_info(path);
        if let Some(ref info) = pe_info {
            details.extend(info.clone());
        }
        // If PE info was empty or unhelpful, try known-exe lookup
        if pe_info.is_none() || pe_info.as_ref().map_or(true, |v| v.is_empty()) {
            if let Some(known) = get_known_executable_info(&name_lower, path) {
                details.extend(known);
            }
        }
    }

    // For .lnk: try to read shortcut target
    if ext == "lnk" {
        if let Some(target) = get_shortcut_target(path) {
            details.push(format!("Target: {}", target));
        }
    }

    // For text-like files: show first meaningful line
    let text_exts = ["txt", "md", "json", "yaml", "yml", "toml", "xml", "ini", "cfg",
                     "log", "csv", "rs", "py", "js", "ts", "tsx", "jsx", "html", "css",
                     "bat", "cmd", "ps1", "sh"];
    if text_exts.contains(&ext.as_str()) {
        if let Ok(content) = fs::read_to_string(file_path) {
            let line_count = content.lines().count();
            details.push(format!("Lines: {}", line_count));

            // For log files, show last meaningful line
            if ext == "log" {
                if let Some(last) = content.lines().rev().find(|l| !l.trim().is_empty()) {
                    let truncated = if last.len() > 80 { &last[..80] } else { last };
                    details.push(format!("Last entry: {}", truncated));
                }
            }
        }
    }

    // Detect project context from parent
    let project = file_path.parent().and_then(|p| detect_project_context(p));

    let file_type = get_file_type(name, &ext, &name_lower);
    let safety_tip = get_safety_tip(&ext, &name_lower, path);

    FileDescription {
        file_type,
        details,
        project,
        safety_tip,
    }
}

fn get_known_executable_info(name_lower: &str, path: &str) -> Option<Vec<String>> {
    let path_lower = path.to_lowercase();
    let mut info = Vec::new();

    // Known executables by name
    match name_lower {
        "cs.exe" | "coursier.exe" => {
            info.push("Coursier — Scala artifact fetcher & launcher".into());
            if path_lower.contains(".metals") || path_lower.contains("metals") {
                info.push("Used by Metals (Scala language server)".into());
            }
        }
        "node.exe" => info.push("Node.js JavaScript runtime".into()),
        "npm.cmd" | "npm.exe" => info.push("Node.js package manager".into()),
        "python.exe" | "python3.exe" => info.push("Python interpreter".into()),
        "pip.exe" | "pip3.exe" => info.push("Python package installer".into()),
        "git.exe" => info.push("Git version control system".into()),
        "code.exe" => info.push("Visual Studio Code editor".into()),
        "cargo.exe" => info.push("Rust package manager & build tool".into()),
        "rustc.exe" => info.push("Rust compiler".into()),
        "rustup.exe" => info.push("Rust toolchain manager".into()),
        "java.exe" | "javaw.exe" => info.push("Java Runtime Environment".into()),
        "javac.exe" => info.push("Java compiler".into()),
        "gradle.exe" | "gradlew.exe" => info.push("Gradle build tool (Java/Kotlin)".into()),
        "mvn.exe" | "mvnw.exe" => info.push("Apache Maven build tool (Java)".into()),
        "docker.exe" => info.push("Docker container engine".into()),
        "kubectl.exe" => info.push("Kubernetes CLI".into()),
        "terraform.exe" => info.push("Infrastructure as Code tool (HashiCorp)".into()),
        "gcloud.exe" => info.push("Google Cloud CLI".into()),
        "az.exe" => info.push("Azure CLI".into()),
        "aws.exe" => info.push("AWS CLI".into()),
        "ffmpeg.exe" => info.push("Audio/video converter & processor".into()),
        "ffprobe.exe" => info.push("Media file analyzer (part of FFmpeg)".into()),
        "7z.exe" | "7za.exe" => info.push("7-Zip archive utility".into()),
        "winrar.exe" => info.push("WinRAR archive utility".into()),
        "powershell.exe" | "pwsh.exe" => info.push("PowerShell command shell".into()),
        "cmd.exe" => info.push("Windows Command Prompt".into()),
        "explorer.exe" => info.push("Windows File Explorer".into()),
        "taskmgr.exe" => info.push("Windows Task Manager".into()),
        "regedit.exe" => info.push("Windows Registry Editor".into()),
        "msiexec.exe" => info.push("Windows Installer".into()),
        "svchost.exe" => info.push("Windows Service Host process".into()),
        "chrome.exe" => info.push("Google Chrome browser".into()),
        "firefox.exe" => info.push("Mozilla Firefox browser".into()),
        "msedge.exe" => info.push("Microsoft Edge browser".into()),
        "spotify.exe" => info.push("Spotify music player".into()),
        "discord.exe" => info.push("Discord chat application".into()),
        "slack.exe" => info.push("Slack messaging application".into()),
        "teams.exe" => info.push("Microsoft Teams".into()),
        "steam.exe" => info.push("Steam gaming platform".into()),
        "notepad++.exe" => info.push("Notepad++ text editor".into()),
        "vlc.exe" => info.push("VLC media player".into()),
        "obs64.exe" | "obs.exe" => info.push("OBS Studio — screen recording & streaming".into()),
        "gimp-2.10.exe" | "gimp.exe" => info.push("GIMP image editor".into()),
        "blender.exe" => info.push("Blender 3D creation suite".into()),
        "unity.exe" => info.push("Unity game engine editor".into()),
        "sbt.exe" | "sbt-launch.jar" => info.push("Scala Build Tool".into()),
        "scala.exe" => info.push("Scala programming language".into()),
        "dotnet.exe" => info.push(".NET CLI".into()),
        "go.exe" => info.push("Go programming language".into()),
        "deno.exe" => info.push("Deno JavaScript/TypeScript runtime".into()),
        "bun.exe" => info.push("Bun JavaScript runtime & bundler".into()),
        _ => {}
    }

    // Context from parent folder
    if info.is_empty() {
        if path_lower.contains("\\node_modules\\.bin\\") || path_lower.contains("/node_modules/.bin/") {
            info.push("Node.js CLI tool (installed via npm)".into());
        } else if path_lower.contains("\\python") && path_lower.contains("\\scripts\\") {
            info.push("Python package executable".into());
        } else if path_lower.contains("\\.cargo\\bin\\") {
            info.push("Rust tool (installed via cargo)".into());
        } else if path_lower.contains("\\scoop\\shims\\") || path_lower.contains("\\scoop\\apps\\") {
            info.push("Installed via Scoop package manager".into());
        }
    }

    if info.is_empty() { None } else { Some(info) }
}

fn get_pe_version_info(path: &str) -> Option<Vec<String>> {
    // Use PowerShell to read .exe/.dll version info
    let mut cmd = std::process::Command::new("powershell");
    cmd.args([
            "-NoProfile", "-Command",
            &format!(
                "$v = (Get-Item '{}').VersionInfo; \
                 Write-Output \"DESC:$($v.FileDescription)\"; \
                 Write-Output \"COMPANY:$($v.CompanyName)\"; \
                 Write-Output \"VERSION:$($v.FileVersion)\"; \
                 Write-Output \"PRODUCT:$($v.ProductName)\"",
                path.replace('\'', "''")
            )
        ]);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    let output = cmd.output().ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut info = Vec::new();

    for line in stdout.lines() {
        if let Some(desc) = line.strip_prefix("DESC:") {
            let desc = desc.trim();
            if !desc.is_empty() { info.push(format!("Description: {}", desc)); }
        } else if let Some(company) = line.strip_prefix("COMPANY:") {
            let company = company.trim();
            if !company.is_empty() { info.push(format!("Publisher: {}", company)); }
        } else if let Some(version) = line.strip_prefix("VERSION:") {
            let version = version.trim();
            if !version.is_empty() { info.push(format!("Version: {}", version)); }
        } else if let Some(product) = line.strip_prefix("PRODUCT:") {
            let product = product.trim();
            if !product.is_empty() { info.push(format!("Product: {}", product)); }
        }
    }

    if info.is_empty() { None } else { Some(info) }
}

fn get_shortcut_target(path: &str) -> Option<String> {
    let mut cmd = std::process::Command::new("powershell");
    cmd.args([
            "-NoProfile", "-Command",
            &format!(
                "$sh = New-Object -ComObject WScript.Shell; \
                 $sc = $sh.CreateShortcut('{}'); \
                 Write-Output $sc.TargetPath",
                path.replace('\'', "''")
            )
        ]);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    let output = cmd.output().ok()?;

    let target = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if target.is_empty() { None } else { Some(target) }
}

fn detect_project_context(dir: &Path) -> Option<String> {
    // Walk up to find project markers (max 5 levels)
    let mut current = Some(dir);
    let mut depth = 0;

    while let Some(p) = current {
        if depth > 5 { break; }

        // Check for project markers
        if p.join("package.json").exists() {
            if let Ok(content) = fs::read_to_string(p.join("package.json")) {
                if let Some(name) = extract_json_field(&content, "name") {
                    return Some(format!("Node.js project: {}", name));
                }
            }
            return Some("Node.js project".into());
        }
        if p.join("Cargo.toml").exists() {
            if let Ok(content) = fs::read_to_string(p.join("Cargo.toml")) {
                for line in content.lines() {
                    if let Some(rest) = line.strip_prefix("name") {
                        if let Some(val) = rest.split('"').nth(1) {
                            return Some(format!("Rust project: {}", val));
                        }
                    }
                }
            }
            return Some("Rust project".into());
        }
        if p.join("pyproject.toml").exists() || p.join("setup.py").exists() {
            return Some("Python project".into());
        }
        if p.join("go.mod").exists() {
            return Some("Go project".into());
        }
        if p.join("pom.xml").exists() {
            return Some("Java/Maven project".into());
        }
        if p.join(".sln").exists() || p.join("*.csproj").to_string_lossy().contains(".csproj") {
            return Some(".NET project".into());
        }

        current = p.parent();
        depth += 1;
    }
    None
}

fn extract_json_field<'a>(json: &'a str, field: &str) -> Option<String> {
    let pattern = format!("\"{}\"", field);
    json.find(&pattern).and_then(|pos| {
        let rest = &json[pos + pattern.len()..];
        let colon = rest.find(':')?;
        let after_colon = rest[colon + 1..].trim_start();
        if after_colon.starts_with('"') {
            let start = 1;
            let end = after_colon[start..].find('"')?;
            Some(after_colon[start..start + end].to_string())
        } else {
            None
        }
    })
}

fn get_folder_type(name: &str) -> String {
    let n = name.to_lowercase();
    match n.as_str() {
        "node_modules" => "Node.js Dependencies".into(),
        ".git" => "Git Repository Data".into(),
        "target" => "Build Output".into(),
        "build" | "dist" | "out" => "Build Output".into(),
        ".vscode" => "VS Code Settings".into(),
        ".idea" => "JetBrains IDE Settings".into(),
        "windows" => "Windows System".into(),
        "$recycle.bin" => "Recycle Bin".into(),
        "system volume information" => "System Restore".into(),
        "program files" | "program files (x86)" => "Installed Programs".into(),
        "documents" => "User Documents".into(),
        "downloads" => "Downloads".into(),
        "pictures" => "User Pictures".into(),
        "videos" => "User Videos".into(),
        "music" => "User Music".into(),
        "desktop" => "Desktop".into(),
        "appdata" => "Application Data".into(),
        ".cache" | "cache" | "temp" | "tmp" => "Cache / Temp".into(),
        "__pycache__" => "Python Cache".into(),
        _ => "Directory".into(),
    }
}

fn get_folder_safety_tip(name: &str) -> Option<String> {
    let n = name.to_lowercase();
    match n.as_str() {
        "node_modules" => Some("Safe to delete — run 'npm install' to restore".into()),
        "target" => Some("Safe to delete — run 'cargo build' to restore".into()),
        "build" | "dist" | "out" => Some("Safe to delete — generated during build".into()),
        "__pycache__" => Some("Safe to delete — Python recreates on next run".into()),
        ".cache" | "cache" | "temp" | "tmp" => Some("Usually safe to delete".into()),
        "windows" | "boot" | "recovery" => Some("⚠️ CRITICAL — do NOT delete!".into()),
        "$recycle.bin" | "system volume information" => Some("⚠️ System managed — do not delete manually".into()),
        "program files" | "program files (x86)" => Some("⚠️ Contains installed software — use uninstaller instead".into()),
        ".git" => Some("Deleting removes all version history!".into()),
        ".ssh" | ".aws" | ".azure" | ".gcloud" => Some("⚠️ Contains credentials — be very careful!".into()),
        "downloads" => Some("Often accumulates old files — good candidate for cleanup".into()),
        _ => None,
    }
}

fn get_file_type(name: &str, ext: &str, name_lower: &str) -> String {
    // Special filenames first
    match name_lower {
        "readme.md" | "readme.txt" | "readme" => return "Project Documentation".into(),
        "license" | "license.md" => return "License File".into(),
        ".gitignore" => return "Git Ignore Rules".into(),
        ".env" | ".env.local" => return "Environment Config".into(),
        "dockerfile" => return "Docker Build File".into(),
        "cargo.toml" => return "Rust Manifest".into(),
        "package.json" => return "Node.js Manifest".into(),
        "tsconfig.json" => return "TypeScript Config".into(),
        _ => {}
    }

    match ext {
        "pdf" => "PDF Document".into(),
        "doc" | "docx" => "Word Document".into(),
        "xls" | "xlsx" => "Excel Spreadsheet".into(),
        "ppt" | "pptx" => "PowerPoint Presentation".into(),
        "txt" => "Text File".into(),
        "md" => "Markdown Document".into(),
        "csv" => "CSV Data".into(),
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" => "Image".into(),
        "mp4" | "avi" | "mkv" | "mov" | "wmv" | "webm" => "Video".into(),
        "mp3" | "wav" | "flac" | "ogg" | "aac" | "m4a" => "Audio".into(),
        "rs" => "Rust Source".into(),
        "py" => "Python Script".into(),
        "js" => "JavaScript".into(),
        "ts" => "TypeScript".into(),
        "tsx" | "jsx" => "React Component".into(),
        "html" | "htm" => "HTML Page".into(),
        "css" => "Stylesheet".into(),
        "json" => "JSON Data".into(),
        "xml" => "XML Data".into(),
        "yaml" | "yml" => "YAML Config".into(),
        "toml" => "TOML Config".into(),
        "sql" => "SQL Script".into(),
        "zip" | "rar" | "7z" | "tar" | "gz" => "Archive".into(),
        "exe" => "Executable".into(),
        "msi" => "Installer".into(),
        "dll" => "Library (DLL)".into(),
        "sys" => "System Driver".into(),
        "bat" | "cmd" => "Batch Script".into(),
        "ps1" => "PowerShell Script".into(),
        "iso" => "Disk Image".into(),
        "lnk" => "Shortcut".into(),
        "log" => "Log File".into(),
        "bak" | "old" => "Backup File".into(),
        "tmp" => "Temporary File".into(),
        _ => {
            if ext.is_empty() {
                format!("File")
            } else {
                format!(".{} File", ext.to_uppercase())
            }
        }
    }
}

fn get_safety_tip(ext: &str, name_lower: &str, path: &str) -> Option<String> {
    let path_lower = path.to_lowercase();

    match ext {
        "tmp" | "log" => Some("Usually safe to delete".into()),
        "bak" | "old" => Some("Backup file — delete if you don't need the backup".into()),
        "iso" | "dmg" | "img" => Some("Disk image — usually safe to delete after use".into()),
        "exe" | "msi" if path_lower.contains("downloads") => {
            Some("Installer in Downloads — safe to delete after installation".into())
        }
        "dll" | "sys" => Some("⚠️ System component — delete only if you know what it is".into()),
        "lnk" => Some("Shortcut only — deleting won't remove the actual program".into()),
        _ => {
            if name_lower.starts_with("~$") {
                Some("Office lock file — safe to delete if the app is closed".into())
            } else {
                None
            }
        }
    }
}

fn format_size_detailed(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;

    if bytes < KB {
        format!("{} bytes", bytes)
    } else if bytes < MB {
        format!("{:.1} KB ({} bytes)", bytes as f64 / KB as f64, format_number(bytes))
    } else if bytes < GB {
        format!("{:.1} MB ({} bytes)", bytes as f64 / MB as f64, format_number(bytes))
    } else {
        format!("{:.2} GB ({} bytes)", bytes as f64 / GB as f64, format_number(bytes))
    }
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { result.push(','); }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn format_time(time: SystemTime) -> String {
    if let Ok(duration) = time.duration_since(SystemTime::UNIX_EPOCH) {
        let secs = duration.as_secs();
        // Simple date formatting
        let days_since_epoch = secs / 86400;
        let years = 1970 + days_since_epoch / 365;
        let remaining_days = days_since_epoch % 365;
        let months = remaining_days / 30 + 1;
        let days = remaining_days % 30 + 1;
        let hours = (secs % 86400) / 3600;
        let minutes = (secs % 3600) / 60;

        // Also compute "X ago" string
        if let Ok(elapsed) = time.elapsed() {
            let elapsed_secs = elapsed.as_secs();
            let ago = if elapsed_secs < 60 {
                "just now".into()
            } else if elapsed_secs < 3600 {
                format!("{} min ago", elapsed_secs / 60)
            } else if elapsed_secs < 86400 {
                format!("{} hours ago", elapsed_secs / 3600)
            } else if elapsed_secs < 86400 * 30 {
                format!("{} days ago", elapsed_secs / 86400)
            } else if elapsed_secs < 86400 * 365 {
                format!("{} months ago", elapsed_secs / (86400 * 30))
            } else {
                format!("{} years ago", elapsed_secs / (86400 * 365))
            };

            format!("{:04}-{:02}-{:02} {:02}:{:02} ({})", years, months, days, hours, minutes, ago)
        } else {
            format!("{:04}-{:02}-{:02} {:02}:{:02}", years, months, days, hours, minutes)
        }
    } else {
        "Unknown".into()
    }
}
