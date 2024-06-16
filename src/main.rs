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
    pub parent: PathVariant,
    pub child: String,
}

// PathKind & PathVariant
#[derive(Clone, Debug)]
pub enum PathKind {
    Standard,
    Environment,
}

#[derive(Clone, Debug)]
pub struct PathVariant {
    pub path: String,
    pub kind: PathKind,
}

pub trait ToEnv {
    fn to_env_path(&self) -> PathBuf;
}

impl ToEnv for ShortcutPath {
    fn to_env_path(&self) -> PathBuf {
        match self.parent.kind {
            PathKind::Standard => {
                let parent = "$".to_owned() + &self.parent.path;
                PathBuf::from(parent).join(PathBuf::from(self.child.clone()))
            },
            PathKind::Environment => {
                PathBuf::from(self.parent.path.clone())
            },
        }
    }
}

pub fn to_bash(fp: &Path, shortcuts: Vec<Shortcut>) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(fp)?;
    file.write_all(b"#!/bin/bash\n\n")?;

    for s in shortcuts {
        let (name, shortcut_path) = (s.name, s.path);
        let line = format!("export {}=\"{}\"\n", name, shortcut_path.to_env_path().display());
        file.write_all(line.as_bytes())?;
    }
    Ok(())
}

pub fn get_home_dir() -> PathBuf {
    dirs::home_dir().expect("$HOME environment variable not initialized.")
}

pub fn get_path_variant(fp: &Path) -> PathKind {
    if dirs::home_dir().is_some() 
        && (fp.starts_with("~")
        || fp.starts_with("$HOME")
        || fp.starts_with("${HOME}")) {
            return PathKind::Environment
    }
    PathKind::Standard
}

pub fn convert_parent_path(fp: &Path, path_kind: PathKind) -> PathVariant {
    let path = match path_kind {
        PathKind::Standard => { fp.parent().unwrap().file_name().unwrap().to_os_string().into_string().unwrap() },
        PathKind::Environment => {
            "~".to_owned()
        }
    };
    PathVariant { path, kind: path_kind }
}

pub fn convert_child_path(fp: &Path) -> String {
    fp.file_name().unwrap().to_os_string().into_string().unwrap()
}

pub fn to_shortcut_paths(folders: Vec<walkdir::DirEntry>) -> Vec<ShortcutPath> {
    let shortcut_paths: Vec<ShortcutPath> = folders
        .into_iter()
        .map(|folder| {
            let fp = folder.into_path();

            // Converting into Shortcut Paths
            // 
            // File Path Type Detections
            // 1. We need to classify different kinds of file paths
            // 2. We need to detect the variants in a cross-platform way
            // 3. We also need to detect these variants of paths in a cross-platform way
            //
            // Solution
            // 1. We classify different paths with an enum PathKind: Environment, Standard
            // 2. We detect the variants using
            //
            // Cases:
            // Linux:
            // - "~/my-folder"
            // - "$HOME/my-folder"
            // - "/home/user/my-folder"

            let path_kind = get_path_variant(&fp);
            println!("Path Kind: {:?}", path_kind);

            let path_variant = convert_parent_path(&fp, path_kind);
            println!("Path Variant: {:?}", path_variant);
            let child = convert_child_path(&fp);
            ShortcutPath { parent: path_variant, child }
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

// Root & ExpandPrefix
#[derive(Clone)]
pub struct Root {
    pub root: PathBuf,
}

pub trait SubstitutePrefix {
    fn starts_with(&self, base: &str) -> bool;
    fn sub_prefix(&self, base: &str, expands_to: String) -> Result<(PathBuf, String), Box<dyn Error>>;
}

impl SubstitutePrefix for Root {
    fn starts_with(&self, base: &str) -> bool {
        self.root.starts_with(base)
    }

    fn sub_prefix(&self, base: &str, replace_with: String) -> Result<(PathBuf, String), Box<dyn Error>> {
        if self.starts_with(base) {
            let children = self.root.strip_prefix(base)?;
            let prefix = PathBuf::from(replace_with);
            let root = prefix.join(children);
            return Ok((root, base.to_string()))
        }
        panic!("Could not substitute prefix");
    }
}

pub fn sub_path(root: &Root, base: &str, replace_with: String) -> Option<(PathBuf, String)> {
    if root.starts_with(base) {
        let root_span = root
            .sub_prefix(base, replace_with)
            .unwrap_or_else(|_| panic!("Could not substitute prefix for {}", root.root.display()));
        return Some(root_span);
    }
    None

}
pub fn expand_home(root: &Root) -> Option<(PathBuf, String)> {
    let expand = |base| sub_path(root, base, get_home_dir().display().to_string());

    expand("~")
        .map_or_else(|| expand("$HOME"), Some)
        .map_or_else(|| expand("${HOME}"), Some)

}

pub fn compact_home(root: &Root, home_prefix: String) -> Option<(PathBuf, String)> {
    sub_path(root, get_home_dir().to_str().unwrap(), home_prefix)
}

pub fn span_path_exists(sp: Option<(PathBuf, String)>) -> bool {
    sp.is_none() || sp.unwrap().0.exists()
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    
    let (root, depth, dest) = (args.root, args.depth, args.dest);
    let root = Root { root };

    // Expand HOME prefixes
    let span_root = expand_home(&root);

    if !span_path_exists(span_root.clone()) {
        eprintln!("{} does not exist.", root.root.display());
        panic!("Root folder does not exist.");
    } 
    let (root, prefix) = span_root.unwrap();

    // Collect all the folders under the root directory
    let folders: Vec<walkdir::DirEntry> = WalkDir::new(root.clone())
        .max_depth(depth)
        .into_iter()
        .skip(1)
        .filter_map(|e| e.ok())
        .filter(|f| f.path().is_dir())
        .collect();

    // NOTE:
    // We skip the root here in our collection since we want to treat
    // expandable paths like root, differently than normal paths.
    // We want to avoid the headache so we intentionally filter these variants out
    // of our homogenous collection

    let root = Root { root };
    let (root, _) = compact_home(&root, prefix).unwrap();

    // Root
    let parent = PathVariant { path: root.display().to_string(), kind: PathKind::Environment };
    let child = convert_child_path(&root);
    let root_shortcut = ShortcutPath { parent, child };

    let mut shortcut_paths = to_shortcut_paths(folders);
    
    shortcut_paths.reverse();
    shortcut_paths.push(root_shortcut);
    shortcut_paths.reverse();

    let shortcuts = to_shortcuts(shortcut_paths);
    
    println!("Creating shortcuts for: ");
    shortcuts.iter().for_each(|s| println!("\t{}", s.path.to_env_path().display()));

    // Convert to bash script
    to_bash(&dest, shortcuts)?;
    println!("Wrote shortcuts to {}", dest.display());
    
    Ok(())
}
