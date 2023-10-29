use std::env;
use std::path::PathBuf;
use std::fmt;
use lazy_static::lazy_static;

use clap::{Arg, Command, value_parser};

lazy_static! {
    static ref HOME_DIR: PathBuf = {
        let home_dir = env::var("HOME").unwrap();
        PathBuf::from(home_dir)
    };
    static ref XDG_DATA_HOME: PathBuf = {
        let xdg_data_home = env::var("XDG_DATA_HOME").unwrap_or_else(|_| {
            let mut xdg_data_home = HOME_DIR.clone();
            xdg_data_home.push(".local/share");
            xdg_data_home.to_str().unwrap().to_string()
        });
        PathBuf::from(xdg_data_home)
    };
    static ref SCRIPTS_DIR: PathBuf = {
        let scripts_dir = XDG_DATA_HOME.join("scripts");
        scripts_dir
    };
}

#[derive(Clone)]
enum AppAbbr {
    Ps,
    Ai,
    Ae,
}

impl AppAbbr {
    /// Creates a new AppAbbr
    /// ```rust
    /// use heyps::AppAbbr;
    /// let app_abbr = AppAbbr::new("ps").unwrap();
    /// assert_eq!(app_abbr, AppAbbr::Ps);
    /// ```
    fn from_str(app_abbr: &str) -> Result<AppAbbr, Box<dyn std::error::Error>> {
        match app_abbr {
            "ps" => Ok(AppAbbr::Ps),
            "ai" => Ok(AppAbbr::Ai),
            "ae" => Ok(AppAbbr::Ae),
            _ => Err("Unsupported application".into()),
        }
    }
    fn expand(&self) -> String {
        match self {
            AppAbbr::Ps => "Photoshop",
            AppAbbr::Ai => "Illustrator",
            AppAbbr::Ae => "After Effects",
        }.to_string()
    }
}


enum AppName {
    Latest(String),
    Beta(String),
    Year(String),
}

impl fmt::Display for AppName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // write!(f, "({}, {})", self.x, self.y)
        match self {
            AppName::Latest(name) => write!(f, "{}", name),
            AppName::Beta(name) => write!(f, "{}", name),
            AppName::Year(name) => write!(f, "{}", name),
        }
    }
}
impl AppName {
    fn new(bundle_id: &String, version_option: &str) -> Result<AppName, Box<dyn std::error::Error>> {
        match version_option {
            "latest" => {
                let query = format!("kMDItemCFBundleIdentifier == \"{}\"", bundle_id);
                let output = std::process::Command::new("mdfind").arg(query).output()?;
                if !output.status.success() {
                    let stderr = std::str::from_utf8(&output.stderr)?;
                    return Err(format!("Failed to execute command: {}", stderr).into());
                }

                let stdout = std::str::from_utf8(&output.stdout)?;
                let name = stdout
                    .lines()
                    .filter(|line| !line.contains("(Beta)"))
                    .last()
                    .map(|line| line.split('/').last().unwrap().to_string())
                    .ok_or("Failed to get the latest version")?
                    .replace(".app", "");
                // Adobe Photoshop 2021
                Ok(AppName::Latest(name))
            }
            "beta" => {
                let query = format!("kMDItemCFBundleIdentifier == \"{}\"", bundle_id);
                let output = std::process::Command::new("mdfind").arg(query).output()?;
                if !output.status.success() {
                    let stderr = std::str::from_utf8(&output.stderr)?;
                    return Err(format!("Failed to execute command: {}", stderr).into());
                }

                let stdout = std::str::from_utf8(&output.stdout)?;
                let error_msg = "Failed to get the beta version";
                let name = stdout
                    .lines()
                    .filter(|line| line.contains("(Beta)"))
                    .last()
                    .map(|line| line.split('/').last().unwrap().to_string()) 
                    .ok_or(error_msg)?
                    .replace(".app", "");
                Ok(AppName::Beta(name))
            }
            year if year.len() == 4 && year.starts_with("20") => {
                let query = format!("kMDItemCFBundleIdentifier == \"{}\"", bundle_id);
                let output = std::process::Command::new("mdfind").arg(query).output()?;
                if !output.status.success() {
                    let stderr = std::str::from_utf8(&output.stderr)?;
                    return Err(format!("Failed to execute command: {}", stderr).into());
                }

                let stdout = std::str::from_utf8(&output.stdout)?;
                let error_msg = "Failed to get the: ".to_string() + year + " version";
                let name = stdout
                    .lines()
                    .filter(|line| !line.contains(year))
                    .last()
                    .map(|line| line.split('/').last().unwrap().to_string())
                    .ok_or(error_msg)?
                    .replace(".app", "");
                Ok(AppName::Year(name))
            }
            _ => Err("Unsupported version".into()),
        }
    }
}

/// Validates a bundle id with: mdfind kMDItemCFBundleIdentifier == "com.adobe.Photoshop"
fn validate_bundle_id(bundle_id: &String) -> Result<(), Box<dyn std::error::Error>> {
    let query = format!("kMDItemCFBundleIdentifier == \"{}\"", bundle_id);
    let output = std::process::Command::new("mdfind").arg(query).output()?;
    if !output.status.success() {
        let stderr = std::str::from_utf8(&output.stderr)?;
        return Err(format!("Failed to execute command: {}", stderr).into());
    }

    let stdout = std::str::from_utf8(&output.stdout)?;
    println!("output: {:?}", output);
    if stdout.lines().count() == 0 {
        return Err("Failed to find the application".into());
    }
    Ok(())
}

fn gen_bundle_id(app_abbr: &AppAbbr) -> Result<String, Box<dyn std::error::Error>> {
    let prefix = "com.adobe.".to_string();
    let suffix = app_abbr.expand().replace(" ", "");
    let bundle_id = prefix + &suffix;
    println!("bundle_id: {}", bundle_id);
    match validate_bundle_id(&bundle_id) {
        Ok(_) => Ok(bundle_id),
        Err(_) => Err("Failed to generate bundle id".into()),
    }
}

struct App {
    name: String,
    // bundle_id: String,
    // version: Version,
}

impl App {
    fn new(app_abbr: AppAbbr, version_option: Option<&String>) -> Result<App, Box<dyn std::error::Error>> {
        let bundle_id = gen_bundle_id(&app_abbr)?;
        let name = AppName::new(&bundle_id, version_option.unwrap_or(&"latest".to_string()))?.to_string();
        Ok(App {
            name,
            // bundle_id,
            // version,
        })
    }

}
enum ScriptType {
    Psjs,
    Jsx,
    Js,
}

impl ScriptType {
    fn new(file_path: &PathBuf) -> Result<ScriptType, Box<dyn std::error::Error>> {
        let extension = file_path.extension().unwrap().to_str().unwrap();
        match extension {
            "psjs" => Ok(ScriptType::Psjs),
            "jsx" => Ok(ScriptType::Jsx),
            "js" => Ok(ScriptType::Js),
            _ => Err("Unsupported file type".into()),
        }
    }
}

struct Script {
    app: App,
    file_path: PathBuf,
    script_type: ScriptType,
}

impl Script {
    fn new(app: App, file_path: &PathBuf) -> Result<Script, Box<dyn std::error::Error>> {
        let script_type = ScriptType::new(&file_path)?;
        Ok(Script {
            app,
            file_path: file_path.to_owned(),
            script_type,
        })
    }

    /// Executes a script using osascript
    #[cfg(target_os = "macos")]
    fn execute_with_osascript(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = self.file_path.to_str().unwrap();
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg(format!(
                "tell application \"{}\" to do javascript of file \"{}\"",
                self.app.name, file_path
            ))
            .output()?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = std::str::from_utf8(&output.stderr)?;
            Err(format!("Failed to execute command: {}", stderr).into())
        }
    }

    /// Executes a script using open with Adobe Photoshop
    #[cfg(target_os = "macos")]
    fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self.script_type {
            ScriptType::Psjs => {
                let output = std::process::Command::new("open")
                    .arg("-a")
                    .arg(&self.app.name)
                    .arg(self.file_path.to_str().unwrap())
                    .output()?;
                if output.status.success() {
                    Ok(())
                } else {
                    let stderr = std::str::from_utf8(&output.stderr)?;
                    Err(format!("Failed to execute command: {}", stderr).into())
                }
            }
            ScriptType::Jsx => {
                Script::execute_with_osascript(&self)
            }
            ScriptType::Js => {
                Script::execute_with_osascript(&self)
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("heyps")
        .version("1.0")
        .author("Oleksii Luchnikov")
        .about("Executes an Adobe script to the target application")
        .arg(
            Arg::new("app")
                .short('a')
                .long("app")
                .value_name("APP")
                .value_parser(["ps", "ai", "ae"])
                .required(true)
                .num_args(1)
                .help("The target Adobe application"),
        )
        .arg(
            Arg::new("target")
                .short('t')
                .value_name("TARGET")
                .default_value("latest")
                .value_parser(["latest", "beta", "year"])
                .help("The target version of the Adobe application"),
        )
        .arg(
            Arg::new("test")
                .short('T')
                .long("test")
                .help("Runs the test script"),
        )
        .arg(
            Arg::new("execute")
                .short('e')
                .long("execute")
                .value_name("FILE_PATH")
                .required(true)
                .value_parser(value_parser!(PathBuf))
                .help("The path to the script file to execute in Adobe Photoshop"),
        )
        .get_matches();

    let script_path: &PathBuf  = matches.get_one::<PathBuf>("execute").unwrap();
    let app = App::new(
        AppAbbr::from_str(matches.get_one::<String>("app").unwrap().as_str())?,
        matches.get_one::<String>("target").as_deref())?;
    let script = Script::new(app, script_path)?;
    script.execute()?;
    Ok(())
}
