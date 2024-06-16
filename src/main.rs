use std::{
    error::Error, 
    fs::File,
    io::Write,
    path::{Path, PathBuf}
};

use clap::Parser;

use shortcut::app::Args;
use walkdir::WalkDir;

pub struct Shortcut {
    pub name: String,
    pub path: ShortcutPath,
}

pub struct ShortcutPath {
    pub parent: String,
    pub child: String,
}

pub trait ToEnv {
    fn to_env_string(&self) -> String;
    fn to_env_path(&self) -> PathBuf;
}

impl ToEnv for ShortcutPath {
    fn to_env_string(&self) -> String {
        let parent = "$".to_owned() + &self.parent;
        let result = PathBuf::from(parent).join(PathBuf::from(self.child.clone()));
        result.into_os_string().into_string().unwrap()
        
    }

    fn to_env_path(&self) -> PathBuf {
        let parent = "$".to_owned() + &self.parent;
        let result = PathBuf::from(parent);
        result.join(PathBuf::from(self.child.clone()))
    }
}

pub fn to_bash(fp: &Path, shortcuts: Vec<Shortcut>) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(fp)?;
    file.write_all(b"#!/bin/bash\n\n")?;

    for s in shortcuts {
        let (name, shortcut_path) = (s.name, s.path);
        let line = format!("export {}=\"{}\"\n", name, shortcut_path.to_env_string());
        file.write_all(line.as_bytes())?;
    }
    Ok(())
}

pub fn to_shortcut_paths(folders: Vec<walkdir::DirEntry>) -> Vec<ShortcutPath> {
    let shortcut_paths: Vec<ShortcutPath> = folders
        .into_iter()
        .map(|folder| {
            let fp = folder.into_path();

            let parent = fp.parent().unwrap().file_name().unwrap().to_os_string().into_string().unwrap();
            let child = fp.file_name().unwrap().to_os_string().into_string().unwrap();

            ShortcutPath { parent, child }
    }).collect();
    shortcut_paths
}

pub fn to_shortcuts(shortcut_paths: Vec<ShortcutPath>) -> Vec<Shortcut> {
    let shortcuts: Vec<Shortcut> = shortcut_paths
        .into_iter()
        .map(|sp| {
            Shortcut { name: sp.child.clone().replace("-", ""), path: sp}
        }).collect();
    shortcuts
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    
    let root = args.root;
    let depth = args.depth;
    let dest = args.dest;

    if !root.exists() {
        eprintln!("{} does not exist.", root.display());
        panic!("Root folder does not exist.");
    }
    
    // Collect all the folders under the root directory
    let folders: Vec<walkdir::DirEntry> = WalkDir::new(root)
        .max_depth(depth)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|f| f.path().is_dir())
        .collect();

    let shortcut_paths = to_shortcut_paths(folders);
    let shortcuts = to_shortcuts(shortcut_paths);
    
    println!("Creating shortcuts for: ");
    shortcuts.iter().for_each(|s| println!("\t{}", s.path.to_env_string()));

    // Convert to bash script
    to_bash(&dest, shortcuts)?;
    println!("Wrote shortcuts to {}", dest.display());
    
    Ok(())
}
