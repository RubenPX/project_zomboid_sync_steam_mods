use ini::Ini;
use std::{
    fs::{self, metadata},
    process::{exit},
};

use std::io;
use std::io::prelude::*;

fn main() {
    // if !admin::is_admin() {
    //     admin::run_as_admin();
    //     exit(0);
    // }

    let [steam_mods_folder, game_mods_folder] = read_ini();
    backup_mods_config(&game_mods_folder);

    match fs::remove_dir_all(&game_mods_folder) {
        Err(err) => println!("err {}", err),
        Ok(_) => fs::create_dir(&game_mods_folder).unwrap(),
    }

    println!("Reading steam mods");

    let mut list_folder_mods: Vec<String> = Vec::new();
    let folders = fs::read_dir(&steam_mods_folder).unwrap();

    for path in folders {
        let mods_path =
            fs::read_dir(path.unwrap().path().display().to_string() + "\\mods").unwrap();
        for mod_p in mods_path {
            list_folder_mods.push(mod_p.unwrap().path().display().to_string());
        }
    }

    println!("Creating commands into \"commands.bat\"");

    let mut cmd_file_data: String = "".to_string();
    for mod_dir in list_folder_mods {
        cmd_file_data = format!(
            "{}\nmklink /D {} {}",
            cmd_file_data,
            format!(
                "\"{}\\{}\"",
                &game_mods_folder,
                mod_dir.split("\\").last().unwrap().to_string()
            ),
            format!("\"{}\"", &mod_dir)
        );
    }
    
    fs::write("./commands.bat", &cmd_file_data).unwrap();

    restore_mods_config(&game_mods_folder);

    println!("Config updated, please run \"commands.bat\" as admin");
    println!("");
    println!("Press enter to exit program");
    pause();
    exit(0);
}

fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we println without a newline and flush manually.
    // write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

fn read_ini() -> [String; 2] {
    let conf;

    match Ini::load_from_file("conf.ini") {
        Ok(ini) => conf = ini,
        Err(_) => {
            create_ini();
            println!("ini file is initialized");
            exit(0);
        }
    }

    let base = conf.section(None::<String>).expect("Failed to read");
    let steam_mods_folder = base
        .get("steam_mods_folder")
        .expect("steam_mods_folder variable not found");
    let game_mods_folder = base
        .get("game_mods_folder")
        .expect("game_mods_folde variable not found");

    if game_mods_folder == "" || steam_mods_folder == "" {
        println!("First you need change variables");
        exit(0);
    }

    return [steam_mods_folder.to_string(), game_mods_folder.to_string()];
}

fn create_ini() {
    let mut conf = Ini::new();
    conf.with_section(None::<String>)
        .set("steam_mods_folder", "\"\"")
        .set("game_mods_folder", "\"\"");
    conf.write_to_file("conf.ini").unwrap();
}

fn get_files(path: &String) -> Result<Vec<String>, io::Error> {
    // let folders = fs::read_dir(path).unwrap();

    let folders = fs::read_dir(path)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect::<Vec<_>>();

    let mut list: Vec<String> = Vec::new();

    for folder in folders {
        let meta = match metadata(&folder) {
            Ok(data) => data,
            Err(_) => {
                // eprintln!("Failed to parse dir: {:?}", folder);
                break;
            }
        };

        if meta.is_file() {
            list.push((folder.into_os_string().into_string().unwrap() as String).replace("\\", "/"))
        }
    }

    return Ok(list);
}

fn backup_mods_config(folder: &String) {
    let mods_files = match get_files(folder) {
        Ok(data) => data,
        Err(_) => {
            eprintln!("Failed to load files from folders");
            exit(1);
        }
    };

    match fs::create_dir("./data/") {
        Ok(_) => {
            println!("Backup data folder created!")
        }
        Err(_) => {
            println!("Backup data folder aready exists")
        }
    }

    for ele in mods_files {
        match fs::copy(&ele, format!("./data/{}", &ele.split("/").last().unwrap())) {
            Ok(_) => {
                println!("Backup ok: {}", &ele)
            }
            Err(_) => {
                println!("Fail backup: {}", &ele);
                exit(-1)
            }
        };
    }
}

fn restore_mods_config(mods_folder: &String) {
    let data_folder = "./data/".to_string();

    let cofig_files = match get_files(&data_folder) {
        Ok(data) => data,
        Err(_) => {
            eprintln!("Failed to load files from backup folder");
            exit(1);
        }
    };

    for ele in cofig_files {
        match fs::copy(
            &ele,
            format!("{}/{}", mods_folder, &ele.split("/").last().unwrap()),
        ) {
            Ok(_) => {
                println!("file restored from backup: {}", &ele)
            }
            Err(_) => {
                println!("Fail to restore file from backup: {}", &ele);
                exit(-1)
            }
        };
    }
}
