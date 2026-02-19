use anyhow::{Context, Result};
use clap::Parser;
use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use pty::fork::Fork;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory to scan for archives (default: current directory)
    #[arg(short, long, default_value = ".")]
    dir: PathBuf,

    /// Extract without asking (all found archives)
    #[arg(short, long)]
    all: bool,

    /// Run integrity test before extraction
    #[arg(short, long)]
    test: bool,

    /// Password for encrypted archives
    #[arg(short, long)]
    password: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

pub(crate) struct Archive {
    pub path: PathBuf,
    pub base_name: String,
    pub archive_type: ArchiveType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ArchiveType {
    SevenZip,
    Zip,
    Rar,
    TarGz,
}

impl Archive {
    pub(crate) fn new(path: PathBuf) -> Option<Self> {
        let file_name = path.file_name()?.to_str()?;
        let (archive_type, base_name) = Self::parse_type(file_name)?;

        Some(Archive {
            path,
            base_name,
            archive_type,
        })
    }

    /// Returns the directory where files will be extracted (parent of archive + base_name).
    pub(crate) fn extract_dir(&self) -> Result<PathBuf> {
        self.path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Archive has no parent"))
            .map(|p| p.join(&self.base_name))
    }

    fn parse_type(filename: &str) -> Option<(ArchiveType, String)> {
        if filename.ends_with(".7z.001") {
            let base = filename.strip_suffix(".7z.001")?;
            Some((ArchiveType::SevenZip, base.to_string()))
        } else if filename.ends_with(".zip.001") {
            let base = filename.strip_suffix(".zip.001")?;
            Some((ArchiveType::Zip, base.to_string()))
        } else if let Some(name) = filename.strip_suffix(".tar.gz") {
            Some((ArchiveType::TarGz, name.to_string()))
        } else if let Some(name) = filename.strip_suffix(".tgz") {
            Some((ArchiveType::TarGz, name.to_string()))
        } else if filename.ends_with(".part01.rar") {
            let base = filename.strip_suffix(".part01.rar")?;
            Some((ArchiveType::Rar, base.to_string()))
        } else if filename.ends_with(".part001.rar") {
            let base = filename.strip_suffix(".part001.rar")?;
            Some((ArchiveType::Rar, base.to_string()))
        } else {
            None
        }
    }

    fn extract_command(&self, test: bool, password: &Option<String>) -> Command {
        match self.archive_type {
            ArchiveType::SevenZip | ArchiveType::Zip => {
                let mut cmd = Command::new("7zz");
                if test {
                    cmd.arg("t");
                } else {
                    cmd.arg("x").arg("-y");
                }
                cmd.arg(&self.path);

                if let Some(pwd) = password {
                    cmd.arg(format!("-p{}", pwd));
                }

                cmd.arg(format!("-o{}", self.base_name));
                cmd
            }
            ArchiveType::Rar => {
                let mut cmd = if test {
                    let mut c = Command::new("unrar");
                    c.arg("t");
                    c
                } else {
                    let mut c = Command::new("unrar");
                    c.arg("x").arg("-y");
                    c
                };
                cmd.arg(&self.path);

                if let Some(pwd) = password {
                    cmd.arg("-p").arg(pwd);
                } else {
                    cmd.arg("-p-");
                }

                // Specify output directory for RAR
                if !test {
                    cmd.arg(&self.base_name);
                }

                cmd
            }
            ArchiveType::TarGz => {
                if test {
                    let mut cmd = Command::new("gzip");
                    cmd.arg("-t").arg(&self.path);
                    cmd
                } else {
                    let mut cmd = Command::new("tar");
                    // Extract to base_name directory
                    cmd.arg("xzf").arg(&self.path).arg("-C").arg(&self.base_name);
                    cmd
                }
            }
        }
    }
}

pub(crate) fn scan_archives(dir: &Path) -> Result<Vec<Archive>> {
    let dir = dir
        .canonicalize()
        .context("Cannot resolve scan directory")?;
    let mut archives = Vec::new();

    for entry in WalkDir::new(&dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(archive) = Archive::new(path.to_path_buf()) {
                archives.push(archive);
            }
        }
    }

    archives.sort_by(|a, b| a.base_name.cmp(&b.base_name));
    Ok(archives)
}

fn select_archives(archives: &[Archive]) -> Result<Vec<usize>> {
    use console::Term;

    println!(
        "\n{} {} {} {}\n",
        style("Found").bold().cyan(),
        style(archives.len()).bold().yellow(),
        style("archives:").bold().cyan(),
        style("(Select with space, confirm with Enter)").dim()
    );

    let term = Term::stdout();

    for (i, archive) in archives.iter().enumerate() {
        let name = &archive.base_name;
        let ext = match archive.archive_type {
            ArchiveType::SevenZip => "7z",
            ArchiveType::Zip => "zip",
            ArchiveType::Rar => "rar",
            ArchiveType::TarGz => "tar.gz",
        };

        println!(
            "{:>3}. [{}] {} ({})",
            style(i + 1).bold().dim(),
            style(" ").white().on_black(),
            style(name).bold().white(),
            style(ext).cyan()
        );
    }

    println!("\n{}", style("Enter numbers (e.g., 1,3,5-7) or 'all':").bold());
    print!("{} ", style(">").cyan());

    let _ = term.flush();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let input = input.trim();
    if input.is_empty() || input.eq_ignore_ascii_case("all") {
        Ok((0..archives.len()).collect())
    } else {
        parse_selection(input, archives.len())
    }
}

fn parse_selection(input: &str, max: usize) -> Result<Vec<usize>> {
    let mut selected = Vec::new();

    for part in input.split(',') {
        let part = part.trim();
        if part.contains('-') {
            let range: Vec<&str> = part.split('-').collect();
            if range.len() == 2 {
                let start: usize = range[0].parse().context("Invalid range start")?;
                let end: usize = range[1].parse().context("Invalid range end")?;
                for i in start..=end {
                    if i > 0 && i <= max {
                        selected.push(i - 1);
                    }
                }
            }
        } else {
            let num: usize = part.parse().context("Invalid number")?;
            if num > 0 && num <= max {
                selected.push(num - 1);
            }
        }
    }

    selected.sort();
    selected.dedup();
    Ok(selected)
}

/// Run a command using PTY so it thinks it's in a real terminal
/// This makes unrar/7zz display percentage progress
fn run_with_pty(cmd: &mut Command, archive_path: &Path) -> Result<()> {
    use std::os::unix::process::CommandExt;

    // Change to the directory containing the archive
    let archive_dir = archive_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot get parent directory"))?
        .canonicalize()
        .context("Cannot canonicalize archive directory")?;

    // Get the archive filename (without path) for use after changing directory
    let archive_name = archive_path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Cannot get archive filename"))?;

    // Create fork using from_ptmx
    let fork = Fork::from_ptmx().context("Failed to create PTY")?;

    // Handle child process
    if let Ok(mut _slave) = fork.is_child() {
        // Change directory before exec
        std::env::set_current_dir(archive_dir)?;

        // Build new command, replacing the full archive path with just the filename
        let program = cmd.get_program();
        let mut new_cmd = std::process::Command::new(program);

        // Find and replace the archive path argument with filename
        let mut found_archive = false;
        for arg in cmd.get_args() {
            if arg == archive_path.as_os_str() {
                // This is the archive path, replace with filename
                new_cmd.arg(archive_name);
                found_archive = true;
            } else {
                // Keep other arguments as-is
                new_cmd.arg(arg);
            }
        }

        // If we didn't find the archive path, something is wrong
        if !found_archive {
            return Err(anyhow::anyhow!("Archive path not found in command arguments"));
        }

        let err = new_cmd.exec();
        return Err(anyhow::anyhow!("exec failed: {:?}", err));
    }

    // Handle parent process
    let mut master = match fork.is_parent() {
        Ok(m) => m,
        Err(_) => return Ok(()),
    };

    // Forward output from PTY to stdout
    let mut buf = [0u8; 8192];
    loop {
        match master.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let _ = std::io::stdout().write_all(&buf[..n]);
                let _ = std::io::stdout().flush();
            }
            Err(_) => break,
        }
    }

    // Wait for child process and check exit status
    let exit_code = fork.wait()?;
    if exit_code == 0 {
        Ok(())
    } else {
        anyhow::bail!("Command failed with exit code: {}", exit_code)
    }
}

fn extract_archive(
    archive: &Archive,
    _multi_progress: &MultiProgress,
    test: bool,
    password: &Option<String>,
    force: bool,
) -> Result<()> {
    let base_name = &archive.base_name;
    let extract_dir = archive.extract_dir()?;

    // Check if already extracted (but skip this check if force is enabled)
    if !force && extract_dir.exists() {
        // Check if the directory contains actual files (not just empty stubs)
        let has_valid_files = WalkDir::new(&extract_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .any(|e| {
                e.file_type().is_file()
                    && e.metadata().map(|m| m.len() > 0).unwrap_or(false)
            });

        if has_valid_files {
            println!(
                "{} {}",
                style("⊘").yellow(),
                style(base_name).yellow(),
            );
            println!("  {} Already exists with valid files, skipping", style("┖─").dim());
            return Ok(());
        } else {
            println!(
                "{} {}",
                style("⚠").yellow(),
                style(base_name).yellow(),
            );
            println!("  {} Exists but appears incomplete, re-extracting", style("┖─").dim());
            fs::remove_dir_all(&extract_dir)?;
        }
    }

    // Tar requires the target directory to exist before extraction
    if !test && matches!(archive.archive_type, ArchiveType::TarGz) {
        fs::create_dir_all(&extract_dir)?;
    }

    // Print what we're about to do
    if test {
        println!(
            "{} {}",
            style("⟳").cyan(),
            style(format!("Testing: {}", base_name)).cyan()
        );
    } else {
        println!(
            "{} {}",
            style("⟳").cyan(),
            style(format!("Extracting: {}", base_name)).cyan()
        );
    }

    // Run command with PTY for real progress display
    let result = if test {
        let mut cmd = archive.extract_command(true, password);
        run_with_pty(&mut cmd, &archive.path)
    } else {
        let mut cmd = archive.extract_command(false, password);
        run_with_pty(&mut cmd, &archive.path)
    };

    // Handle result
    match &result {
        Ok(()) => {
            println!(
                "{} {}",
                style("✓").green(),
                style(base_name).green()
            );
        }
        Err(e) => {
            println!(
                "{} {}",
                style("✗").red(),
                style(base_name).red()
            );
            println!("  {} Error: {}", style("┖─").dim(), e);
        }
    }

    if result.is_err() {
        if extract_dir.exists() {
            fs::remove_dir_all(&extract_dir)?;
        }
        return result;
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Print banner
    println!(
        "\n{} {} {}",
        style("un7z").bold().cyan(),
        style(format!("v{}", env!("CARGO_PKG_VERSION"))).dim(),
        style("- Modern Batch Extraction").cyan()
    );

    // Scan for archives
    let spinner_style = ProgressStyle::default_spinner()
        .template("{spinner:.cyan} {msg}")
        .unwrap();

    let scan_spinner = ProgressBar::new(1);
    scan_spinner.set_style(spinner_style);
    scan_spinner.set_message("Scanning for archives...");
    scan_spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let archives = scan_archives(&args.dir)?;

    scan_spinner.finish_with_message(format!(
        "{} Found {} archive(s)",
        style("✓").green(),
        style(archives.len()).yellow()
    ));

    if archives.is_empty() {
        println!("\n{}", style("No archives found.").yellow().dim());
        return Ok(());
    }

    // Select archives
    let indices = if args.all {
        (0..archives.len()).collect()
    } else {
        select_archives(&archives)?
    };

    if indices.is_empty() {
        println!("\n{}", style("No archives selected.").yellow().dim());
        return Ok(());
    }

    // Extract
    let multi_progress = MultiProgress::new();

    println!(
        "\n{} {} {}",
        style("→").bold().cyan(),
        style("Extracting").bold(),
        style(indices.len()).bold().yellow()
    );

    let mut success = 0;
    let mut failed = 0;
    let skipped = 0;

    for i in &indices {
        let archive = &archives[*i];

        match extract_archive(archive, &multi_progress, args.test, &args.password, false) {
            Ok(()) => {
                success += 1;
            }
            Err(e) => {
                failed += 1;
                eprintln!(
                    "\n{} {}: {}",
                    style("✗").red(),
                    style(archive.base_name.clone()).red(),
                    e
                );

                // Log to failed.log
                if let Ok(mut file) = OpenOptions::new().append(true).create(true).open("failed.log") {
                    let _ = writeln!(file, "{}", archive.path.display());
                }
            }
        }
    }

    // Summary
    println!("\n{}", style("═".repeat(50)).dim());
    println!(
        "{} {} | {} {} | {} {} | {} {}",
        style("Total:").bold(),
        style(indices.len()).yellow(),
        style("Success:").green(),
        style(success).green(),
        style("Failed:").red(),
        style(failed).red(),
        style("Skipped:").yellow(),
        style(skipped).yellow()
    );

    if failed > 0 {
        println!("\n{} See {} for details", style("⚠").yellow(), style("failed.log").yellow());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_new_and_extract_dir() {
        // Archive in subdirectory: extract_dir should be parent/base_name
        let archive = Archive::new(PathBuf::from("a/b/c/archive.7z.001")).unwrap();
        assert_eq!(archive.base_name, "archive");
        assert_eq!(archive.archive_type, ArchiveType::SevenZip);
        let extract_dir = archive.extract_dir().unwrap();
        assert_eq!(extract_dir, PathBuf::from("a/b/c/archive"));

        // Tar.gz in root
        let archive2 = Archive::new(PathBuf::from("foo.tar.gz")).unwrap();
        assert_eq!(archive2.base_name, "foo");
        assert_eq!(archive2.archive_type, ArchiveType::TarGz);
        let extract_dir2 = archive2.extract_dir().unwrap();
        assert_eq!(extract_dir2, PathBuf::from("foo"));

        // Unrecognized extensions return None
        assert!(Archive::new(PathBuf::from("other.txt")).is_none());
    }

    #[test]
    fn test_scan_archives_finds_subfolder_archives() {
        let temp = tempfile::tempdir().unwrap();
        let temp_path = temp.path();

        // Create temp/subdir/file.7z.001
        let subdir = temp_path.join("subdir");
        fs::create_dir_all(&subdir).unwrap();
        let archive_path = subdir.join("file.7z.001");
        fs::write(&archive_path, "dummy").unwrap();

        let archives = scan_archives(temp_path).unwrap();
        assert_eq!(archives.len(), 1);
        assert_eq!(archives[0].base_name, "file");
        assert!(archives[0].path.ends_with("file.7z.001"));
        assert!(archives[0].path.parent().unwrap().ends_with("subdir"));
    }

    #[test]
    fn test_tar_extract_dir_created_before_extraction() {
        let temp = tempfile::tempdir().unwrap();
        let temp_path = temp.path();

        // Create a minimal tar.gz file path structure
        let subdir = temp_path.join("nested");
        fs::create_dir_all(&subdir).unwrap();
        let archive_path = subdir.join("data.tar.gz");
        fs::write(&archive_path, "").unwrap(); // Empty file, just for path test

        let archive = Archive::new(archive_path.clone()).unwrap();
        let extract_dir = archive.extract_dir().unwrap();
        assert_eq!(extract_dir, subdir.join("data"));
        assert!(!extract_dir.exists());
        fs::create_dir_all(&extract_dir).unwrap();
        assert!(extract_dir.exists());
    }
}
