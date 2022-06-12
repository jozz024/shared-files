#![feature(proc_macro_hygiene)]

use arcropolis_api::*;
use std::path::{Path, PathBuf};
mod config;
use config::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub enum Event {
    ArcFilesystemMounted,
    ModFilesystemMounted,
}

pub type EventCallbackFn = extern "C" fn(Event);

extern "C" {
    fn arcrop_register_event_callback(ty: Event, callback: EventCallbackFn);
}


#[arc_callback]
fn normal_callback(hash: u64, data: &mut [u8]) -> Option<usize> {
    match SHARED_FILES.lock().unwrap().get(&hash) {
        Some(info) => {
            match std::fs::read(info.path.to_path_buf()) {
                Ok(file) => {
                    data[..file.len()].copy_from_slice(&file);
                    Some(file.len())
                }
                Err(_err) => {
                    match std::fs::read(&info.fuse_path) {
                        Ok(fuse_file) => {
                            data[..fuse_file.len()].copy_from_slice(&fuse_file);
                            Some(fuse_file.len())
                        }
                        Err(_err) => {
                            load_original_file(&info.hash, data)
                        }
                    }
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

pub extern "C" fn main_real(event: Event) {
    get_configs();

    for (k, v) in SHARED_FILES.lock().unwrap().iter() {
        match v.section {
            Section::Normal => normal_callback::install(*k, v.size),
            Section::Stream => stream_callback::install(*k)
        }
    }
}

fn get_configs() {
    read_from_umm_path(Path::new("sd:/ultimate/mods"));
    println!("[Shared Files::get_configs] Finished reading UMM path!");
}

#[skyline::main(name = "share-files")]
pub fn main() {
    unsafe {
        arcrop_register_event_callback(Event::ArcFilesystemMounted, main_real);
    }

}
