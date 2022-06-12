use serde::Deserialize;
use std::sync::Mutex;
use std::{collections::HashMap, fs, fs::{metadata}, path::{Path, PathBuf}};
use arcropolis_api::*;

lazy_static::lazy_static! {
    pub static ref SHARED_FILES: Mutex<HashMap<u64, Info>> = Mutex::new(HashMap::new());
}

#[derive(Debug)]
pub enum Section {
    Normal,
    Stream,
}

#[derive(Debug)]
pub struct Info {
    pub size: usize,
    pub hash: Hash40,
    pub fuse_path: String,
    pub path: PathBuf,
    pub section: Section
}

#[derive(Deserialize, Debug)]
pub struct SharedFilesConfig {
    pub files: HashMap<String, Vec<String>>,
}

pub fn read_from_umm_path(path: &Path) {
    match fs::read_dir(&path) {
        Ok(res) => {
            for entry in res {
                let entry = entry.unwrap();
                let entry_str = format!("{}", entry.path().display());
                let mut entry_path = path.to_path_buf();
                entry_path.push(entry.path());
                println!("[Share Files] Path: {}", entry_str);
                
                // Ignore anything that starts with a period
                if entry_path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with(".")
                    ||
                    !is_mod_enabled(Hash40::from(entry_str.as_str()).as_u64())
                {
                    continue;
                }
                

                entry_path.push("share.toml");

                if fs::metadata(&entry_path).is_ok() {
                    println!(
                        "[Shared Files::read_from_umm_path] {}",
                        entry_path.display()
                    );
                    match fs::read_to_string(&entry_path) {
                        Ok(content) => {
                            entry_path.pop(); // Remove share.toml
                            add_to_config(content, &entry_path);
                        }
                        Err(_) => {}
                    };
                }
            }
        }
        Err(_) => println!(
            "[Shared Files::read_from_umm_path] Path {} does not exist!",
            path.display()
        ),
    }
}

// pub fn read_from_rom_path(path: &Path) {
//     match fs::read_to_string(path) {
//         Ok(res) => {
//             let mut parent_path = path.to_path_buf();
//             parent_path.pop();
//             add_to_config(res, &parent_path);
//         },
//         Err(_) => println!(
//             "[Shared Files::read_from_rom_path] Failed to read {}",
//             path.display()
//         ),
//     }
// }

fn add_to_config(content: String, path: &PathBuf) {
    let config: SharedFilesConfig = match toml::from_str(&content) {
        Ok(s) => s,
        Err(err) => {
            println!(
                "[Shared Files::add_to_config] Failed parsing content! Reason: {}",
                err
            );
            return;
        }
    };

    for (k, v) in config.files.iter() {        
        println!("{}", k);
        // let k = k.replace("stream:", "stream;").replace("prebuilt:", "prebuilt;"); // File that will be loaded
        let k = k.replace(":", ";"); // File that will be loaded
        
        for i in v {
            // let i = i.replace("stream;", "stream:").replace("prebuilt;", "prebuilt:"); // File that will be hooked
            let i = i.replace(";", ":"); // File that will be hooked
            
            let mut file_path = path.to_path_buf();
            file_path.push(&k);


            SHARED_FILES.lock().unwrap().insert(Hash40::from(&i[..]).as_u64(), Info {
                size: get_file_size(&file_path).unwrap_or(get_file_size(&Path::new(&format!("arc:/{}", k)).to_path_buf()).unwrap()),
                hash: Hash40::from(&k[..]),
                fuse_path: format!("arc:/{}", k),
                path: file_path,
                section: {
                    if i.contains("stream:") {
                        Section::Stream
                    }else {
                        Section::Normal
                    }
                }
            });
        }
    }
}

pub fn get_file_size(path: &PathBuf) -> Option<usize> {
    let md = metadata(&path);

    match md {
        Ok(info) => Some(info.len() as usize),
        Err(_err) => None
    }
}
