#![windows_subsystem = "windows"]
use config::{Config, File};
use simple_log::{error, info, log_level, LogConfigBuilder};
use std::{
    error::Error,
    fs,
    path::Path,
    process::{Child, Command, ExitCode},
    sync::Mutex,
};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(not(debug_assertions))]
use special_folder::SpecialFolder;

#[cfg(not(debug_assertions))]
use mslnk::ShellLink;

#[cfg(not(debug_assertions))]
use std::env;

static CONFIG_FILE: &str = "config.toml";
static DEFAULT_CONFIG: &str = r#"
[process.example]
path = 'C:\Example\Path\To\file.exe'
args = "-e -x -a -m -p -l -e"
hide = false

[process.another_example]
path = 'C:\Example2\Path\To\file.exe'
args = ""
hide = true
"#;
static CHILD_LIST_LOCK: Mutex<Vec<Child>> = Mutex::new(vec![]);

#[cfg(not(debug_assertions))]
fn add_to_startup(display_name: &str, path: &str) -> Result<(), Box<dyn Error>> {
    let startup_path = SpecialFolder::Startup.get().unwrap_or("".into());

    if startup_path.as_os_str().is_empty() {
        return Err("Couldn't get the startup path.".into());
    }

    let shortcut = ShellLink::new(path)?;

    if let Err(err) = shortcut.create_lnk(startup_path.join(format!("{}.lnk", display_name))) {
        Err(err.into())
    } else {
        Ok(())
    }
}

fn spawn_process(path: &str, args: &str, is_hidden: bool) -> Result<(), Box<dyn Error>> {
    let working_dir = if let Some(val) = Path::new(path).parent() {
        val
    } else {
        return Err("Couldn't get the directory to run the process in.".into());
    };

    let mut command = Command::new(path);
    let mut cmd = command
        .current_dir(working_dir)
        .args(args.split(" ").filter(|x| !x.is_empty()));

    #[cfg(target_os = "windows")]
    {
        if is_hidden {
            cmd = cmd.creation_flags(0x08000000);
        }
    }

    let child = cmd.spawn()?;

    let mut child_list = CHILD_LIST_LOCK.lock()?;
    child_list.push(child);

    Ok(())
}

fn _main() -> Result<(), Box<dyn Error>> {
    simple_log::new(
        LogConfigBuilder::builder()
            .path("logs.txt")
            .level(log_level::INFO)
            .size(1)
            .roll_count(1)
            .output_file()
            .build(),
    )?;

    #[cfg(not(debug_assertions))]
    {
        let bin_path = env::current_exe()?;
        let bin_path = bin_path.to_str().unwrap_or("");

        if !bin_path.is_empty() {
            add_to_startup("Process Runner", bin_path)?;
        } else {
            error!("Binary path is empty. Couldn't set the file as auto launch.");
        }
    }

    if !Path::new(CONFIG_FILE).exists() {
        fs::write(CONFIG_FILE, DEFAULT_CONFIG.trim())?;
    }

    let config = Config::builder()
        .add_source(File::with_name(CONFIG_FILE))
        .build()?;

    let process_list = config.get_table("process")?;

    for (key, val) in process_list.iter() {
        let process_name = key;
        let process = val.to_owned().into_table()?;

        info!("Spawning the process \"{}\"...", process_name);

        let path = if let Some(val) = process.get("path") {
            val
        } else {
            error!("Couldn't get the process \"{}\" path.", process_name);
            continue;
        };

        let args = if let Some(val) = process.get("args") {
            val
        } else {
            error!("Couldn't get the process \"{}\" args.", process_name);
            continue;
        };

        let is_hidden = if let Some(val) = process.get("hide") {
            val.to_owned().into_bool().unwrap_or(false)
        } else {
            false
        };

        let result = spawn_process(
            path.to_string().as_str(),
            args.to_string().as_str(),
            is_hidden,
        );

        if let Err(err) = result {
            error!(
                "Couldn't spawn the process \"{}\". Error: {}",
                process_name,
                err.to_string()
            );
        } else {
            info!("Process \"{}\" is successfully spawned.", process_name);
        };
    }

    let mut child_list = CHILD_LIST_LOCK.lock()?;

    for child in child_list.iter_mut() {
        let _ = child.wait();
    }

    info!("All child processes are exited. Terminating the program...");
    Ok(())
}

fn main() -> ExitCode {
    if let Err(err) = _main() {
        error!("{}", err.to_string());
        return 1.into();
    }

    0.into()
}
