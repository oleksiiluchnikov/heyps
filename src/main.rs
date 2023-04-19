use std::error::Error;
use clap::{Arg, Command};

enum ScriptType {
    Psjs,
    Jsx,
    Js,
}

/// Gets the script type based on the file extension
fn get_script_type(file_path: &str) -> Result<ScriptType, Box<dyn std::error::Error>> {
    match file_path {
        file_path if file_path.ends_with(".psjs") => Ok(ScriptType::Psjs),
        file_path if file_path.ends_with(".jsx") => Ok(ScriptType::Jsx),
        file_path if file_path.ends_with(".js") => Ok(ScriptType::Js),
        _ => Err("Unsupported file type".into()),
    }
}

/// Executes a script using osascript
#[cfg(target_os = "macos")]
fn execute_with_osascript(app_name: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(format!("tell application \"{}\" to do javascript of file \"{}\"",app_name, file_path))
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
fn execute_ps_script(app_name: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let script_type = get_script_type(file_path)?;
    match script_type {
        ScriptType::Psjs => {
            let output = std::process::Command::new("open")
                .arg("-a")
                .arg(app_name)
                .output()?;
            if output.status.success() {
                Ok(())
            } else {
                let stderr = std::str::from_utf8(&output.stderr)?;
                Err(format!("Failed to execute command: {}", stderr).into())
            }
        }
        ScriptType::Jsx => {
            execute_with_osascript(app_name, file_path)
        }
        ScriptType::Js => {
            execute_with_osascript(app_name, file_path)
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("heyps")
        .version("1.0")
        .author("Oleksii Luchnikov")
        .about("Executes an Adobe Photoshop script")
        .arg(Arg::new("execute")
            .short('e')
            .long("execute")
            .value_name("FILE_PATH")
            .required(true)
            .help("The path to the script file to execute in Adobe Photoshop"))
        .get_matches();

    let app_name = "Adobe Photoshop 2023";
    let file_path = matches.get_one::<String>("execute").expect("Failed to get file path");
    execute_ps_script(&app_name, &file_path)?;
    Ok(())
}
