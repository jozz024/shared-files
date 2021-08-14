#![feature(proc_macro_hygiene)]

use arcropolis_api::*;
use std::{path::{Path, PathBuf}, collections::HashMap};
use toml::Value;
mod config;
use config::*;

#[arc_callback]
fn normal_callback(hash: u64, data: &mut [u8]) -> Option<usize> {
    println!("{:#x}", hash);
    match SHARED_FILES.lock().unwrap().get(&hash) {
        Some(info) => {
            match std::fs::read(info.path.to_path_buf()) {
                Ok(file) => {
                    data[..file.len()].copy_from_slice(&file);
                    Some(file.len())
                }
                Err(_err) => {
                    load_original_file(hash, data)
                }
            }
        },
        None => load_original_file(hash, data)
    }
}

#[stream_callback]
fn stream_callback(hash: u64) -> Option<PathBuf> {
    match SHARED_FILES.lock().unwrap().get(&hash) {
        Some(info) => Some(info.path.to_path_buf()),
        None => None
    }
}

fn get_configs() {
    match std::fs::read_to_string("sd:/atmosphere/contents/01006A800016E000/romfs/arcropolis.toml")
    {
        Ok(content) => {
            match content.parse::<Value>().unwrap()["paths"]["umm"].as_str() {
                Some(res) => {
                    read_from_umm_path(Path::new(&res.to_string()));
                    println!("[Shared Files::main] Finished reading UMM path!");
                }
                None => println!("[Shared Files::main] Failed parsing ARCropolis config file (UMM Path)!"),
            }
            
            match content.parse::<Value>().unwrap()["paths"]["arc"].as_str() {
                Some(res) => {
                    let mut path = PathBuf::from(&res.to_string());
                    path.push("share.toml");
                    read_from_rom_path(&Path::new(&path));
                    println!("[Shared Files::main] Finished reading ARC path!");
                }
                None => println!("[Shared Files::main] Failed parsing ARCropolis config file (ARC Path)!"),
            }
        } 
        Err(_) => println!("[Shared Files::main] ARCropolis file doesn't exist or failed to parse correctly!"),
    };
}

#[skyline::main(name = "share-files")]
pub fn main() {
    get_configs();

    for (k, v) in SHARED_FILES.lock().unwrap().iter() {
        match v.section {
            Section::Normal => normal_callback::install(*k, v.size),
            Section::Stream => stream_callback::install(*k)
        }
        println!("{:#x} -> {:?}", k, v);
    }
}