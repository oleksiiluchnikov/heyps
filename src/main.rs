use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use clap::{builder::ValueParser, Arg, ArgAction, Command};

/// Represents the type of a script file based on its extension.
///
/// Supported types:
/// - Psjs: A Photoshop JavaScript file.
/// - Jsx: A JavaScript file for Adobe applications.
/// - Js: A plain JavaScript file.
#[derive(Clone, Debug)]
enum ScriptType {
    Psjs,
    Jsx,
    Js,
}

impl ScriptType {
    /// Determines the script type based on the file path's extension.
    ///
    /// # Arguments
    /// * `file_path` - A reference to the path of the script file.
    ///
    /// # Errors
    /// Returns an error if the file does not have a valid extension
    /// or if the extension is not supported.
    fn from_path(file_path: &Path) -> Result<ScriptType, Box<dyn Error>> {
        let ext = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or("The script file does not have an extension")?
            .to_lowercase();
        match ext.as_str() {
            "psjs" => Ok(ScriptType::Psjs),
            "jsx" => Ok(ScriptType::Jsx),
            "js" => Ok(ScriptType::Js),
            _ => Err("Unsupported file type (supported: .psjs, .jsx, .js)".into()),
        }
    }
}

/// Target version of the application
///
/// Latest - latest version of the application
/// Beta - beta version of the application
/// Year(u16) - year of the application release
#[derive(Clone, Debug)]
enum TargetVersion {
    Latest,
    Beta,
    Year(u16),
}

fn parse_target(s: &str) -> Result<TargetVersion, String> {
    match s {
        "latest" => Ok(TargetVersion::Latest),
        "beta" => Ok(TargetVersion::Beta),
        _ => {
            if s.len() == 4 && s.starts_with("20") {
                s.parse::<u16>()
                    .map(TargetVersion::Year)
                    .map_err(|_| format!("Invalid year: {}", s))
            } else {
                Err(format!("Invalid target: {}. Use: latest, beta, or 20XX", s))
            }
        }
    }
}

/// Abbreviation of the application
/// e.g. ps, ai, ae
#[derive(Clone, Debug, PartialEq, Eq)]
enum AppAbbr {
    Ps,
    Ai,
    Ae,
}

/// Parse an application abbreviation from a string
/// Get the base display name of the application
impl FromStr for AppAbbr {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ps" => Ok(AppAbbr::Ps),
            "ai" => Ok(AppAbbr::Ai),
            "ae" => Ok(AppAbbr::Ae),
            _ => Err("Unsupported application. Use: ps|ai|ae".into()),
        }
    }
}

impl AppAbbr {
    /// Get the base display name of the application
    fn base_display_name(&self) -> &'static str {
        match self {
            AppAbbr::Ps => "Adobe Photoshop",
            AppAbbr::Ai => "Adobe Illustrator",
            AppAbbr::Ae => "Adobe After Effects",
        }
    }

    /// Get the bundle ID of the application
    fn bundle_id(&self) -> &'static str {
        // Use actual CFBundleIdentifier values (case sensitive)
        match self {
            AppAbbr::Ps => "com.adobe.Photoshop",
            AppAbbr::Ai => "com.adobe.Illustrator",
            AppAbbr::Ae => "com.adobe.AfterEffects",
        }
    }
}

/// Represents an application
struct App {
    /// Application abbreviation
    abbr: AppAbbr,
    /// Bundle ID of the application
    bundle_id: String,
    /// Name of the application
    name: String, // e.g. "Adobe Photoshop 2024" or "Adobe Photoshop (Beta)"
    // Full .app path
    path: PathBuf, // full .app path
    /// Target version of the application
    target: TargetVersion,
}

/// Implements the Display trait for the App struct
impl fmt::Display for App {
    /// Formats the App struct for display
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.target {
            TargetVersion::Latest => write!(f, "{} [latest]", self.name),
            TargetVersion::Beta => write!(f, "{} [beta]", self.name),
            TargetVersion::Year(y) => write!(f, "{} [{}]", self.name, y),
        }
    }
}

#[cfg(target_os = "macos")]
/// Find all apps with the given bundle ID
fn mdfind_apps(bundle_id: &str) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let query = format!("kMDItemCFBundleIdentifier == \"{}\"", bundle_id);
    let output = std::process::Command::new("mdfind").arg(query).output()?;
    if !output.status.success() {
        return Err(format!("mdfind failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    let paths: Vec<PathBuf> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(PathBuf::from)
        .filter(|p| p.extension().map(|e| e == "app").unwrap_or(false))
        .collect();
    Ok(paths)
}

#[cfg(target_os = "macos")]
/// Choose the app path based on the target version
fn choose_app_path(paths: &[PathBuf], target: &TargetVersion) -> Option<PathBuf> {
    // Sort deterministically
    let mut paths = paths.to_vec();
    paths.sort();

    let is_beta = |p: &PathBuf| {
        p.file_name()
            .map(|n| n.to_string_lossy().contains("(Beta)"))
            .unwrap_or(false)
    };

    match target {
        TargetVersion::Latest => {
            // Prefer non-beta, fallback to any
            let latest_non_beta = paths.iter().filter(|p| !is_beta(p)).last().cloned();
            latest_non_beta.or_else(|| paths.last().cloned())
        }
        TargetVersion::Beta => paths.iter().filter(|p| is_beta(p)).last().cloned(),
        TargetVersion::Year(y) => {
            let y = y.to_string();
            paths.into_iter().rev().find(|p| {
                p.file_name()
                    .map(|n| n.to_string_lossy().contains(&y))
                    .unwrap_or(false)
            })
        }
    }
}

#[cfg(target_os = "macos")]
/// Create a new App struct
impl App {
    /// Create a new App struct
    fn new(abbr: AppAbbr, target: TargetVersion) -> Result<Self, Box<dyn Error>> {
        let bundle_id = abbr.bundle_id().to_string();
        let candidates = mdfind_apps(&bundle_id)?;
        if candidates.is_empty() {
            return Err(format!(
                "{} not found (bundle id: {})",
                abbr.base_display_name(),
                bundle_id
            )
            .into());
        }
        let chosen =
            choose_app_path(&candidates, &target).ok_or("Requested target version not found")?;
        let name = chosen
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .ok_or("Failed to determine app name")?;
        Ok(App {
            abbr,
            bundle_id,
            name,
            path: chosen,
            target,
        })
    }
}

#[cfg(target_os = "macos")]
fn escape_applescript_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Represents a script
struct Script {
    /// Application to execute the script
    app: App,
    /// Path to the script file
    file_path: PathBuf,
    /// Type of the script file
    script_type: ScriptType,
    /// Verbose mode
    verbose: bool,
}

impl Script {
    /// Create a new Script struct
    fn new(app: App, file_path: &Path, script_type: ScriptType, verbose: bool) -> Script {
        Script {
            app,
            file_path: file_path.to_owned(),
            script_type,
            verbose,
        }
    }

    #[cfg(target_os = "macos")]
    /// Run a command and print the output if verbose mode is enabled
    /// Executes a script using osascript
    /// Executes a script using `open -a` (useful for .psjs in Photoshop)
    fn run_cmd(&self, mut cmd: std::process::Command) -> Result<(), Box<dyn Error>> {
        let output = cmd.output()?;
        if self.verbose {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stdout.trim().is_empty() {
                eprintln!("[heyps] stdout:\n{}", stdout);
            }
            if !stderr.trim().is_empty() {
                eprintln!("[heyps] stderr:\n{}", stderr);
            }
        }
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "Command failed with status {}",
                output.status.code().unwrap_or(-1)
            )
            .into())
        }
    }

    /// Executes a script using osascript
    #[cfg(target_os = "macos")]
    /// Executes a script using osascript
    fn execute_with_osascript(&self) -> Result<(), Box<dyn Error>> {
        let path = self.file_path.to_string_lossy();
        let escaped = escape_applescript_string(&path);

        // Choose AppleScript command per app
        let tell_cmd = match self.app.abbr {
            AppAbbr::Ps | AppAbbr::Ai => format!(
                "tell application \"{}\" to do javascript of (POSIX file \"{}\")",
                self.app.name, escaped
            ),
            AppAbbr::Ae => {
                if matches!(self.script_type, ScriptType::Js) {
                    return Err("After Effects does not support plain .js; use .jsx".into());
                }
                format!(
                    "tell application \"{}\" to DoScriptFile (POSIX file \"{}\")",
                    self.app.name, escaped
                )
            }
        };

        let mut cmd = std::process::Command::new("osascript");
        cmd.arg("-e").arg(tell_cmd);
        self.run_cmd(cmd)
    }

    /// Executes a script using `open -a` (useful for .psjs in Photoshop)
    #[cfg(target_os = "macos")]
    fn execute_with_open(&self) -> Result<(), Box<dyn Error>> {
        let mut cmd = std::process::Command::new("open");
        // Use the resolved full .app path to disambiguate versions
        cmd.arg("-a").arg(&self.app.path).arg(&self.file_path);
        self.run_cmd(cmd)
    }

    /// Executes the script based on its type and target application.
    #[cfg(target_os = "macos")]
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        match self.script_type {
            ScriptType::Psjs => {
                if !matches!(self.app.abbr, AppAbbr::Ps) {
                    return Err(".psjs is only supported by Adobe Photoshop".into());
                }
                self.execute_with_open()
            }
            ScriptType::Jsx | ScriptType::Js => self.execute_with_osascript(),
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn main() -> Result<(), Box<dyn Error>> {
    eprintln!("heyps currently supports macOS only.");
    std::process::exit(1);
}

#[cfg(target_os = "macos")]
fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("heyps")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Execute Adobe app scripts from the terminal")
        .arg(
            Arg::new("app")
                .short('a')
                .long("app")
                .value_name("APP")
                .required(true)
                .default_value("ps")
                .value_parser(["ps", "ai", "ae"])
                .help("Target Adobe application: ps|ai|ae"),
        )
        .arg(
            Arg::new("target")
                .short('t')
                .long("target")
                .value_name("TARGET")
                .default_value("latest")
                .value_parser(ValueParser::new(parse_target))
                .help("Target version: latest, beta, or 20XX (e.g., 2024)"),
        )
        .arg(
            Arg::new("execute")
                .short('e')
                .long("execute")
                .value_name("FILE_PATH")
                .required(true)
                .help("Path to the script file (.psjs, .jsx, .js)"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Verbose output"),
        )
        .arg(
            Arg::new("test")
                .short('T')
                .long("test")
                .action(ArgAction::SetTrue)
                .hide(true) // keep for future use, hidden for now
                .help("Runs the test script (reserved)"),
        )
        .get_matches();

    let file_path = PathBuf::from(matches.get_one::<String>("execute").unwrap());
    if !file_path.exists() {
        return Err("The script file does not exist".into());
    }
    if !file_path.is_file() {
        return Err("The script should be a file".into());
    }

    let script_type = ScriptType::from_path(&file_path)?;
    let app_abbr = matches
        .get_one::<String>("app")
        .unwrap()
        .parse::<AppAbbr>()
        .map_err(|e| format!("Invalid app: {}", e))?;
    let target = matches
        .get_one::<TargetVersion>("target")
        .expect("defaulted")
        .clone();
    let verbose = matches.get_flag("verbose");

    let app = App::new(app_abbr, target)?;
    let script = Script::new(app, &file_path, script_type, verbose);

    if verbose {
        eprintln!("[heyps] Using app: {}", script.app);
        eprintln!("[heyps] App path: {}", script.app.path.display());
        eprintln!("[heyps] Script: {}", script.file_path.display());
    }

    script.execute()?;
    Ok(())
}
