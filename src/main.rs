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
    pub parent: String,
    pub child: String,
    pub kind: PathKind,
}

// PathKind
#[derive(Clone, Debug)]
pub enum PathKind {
    Standard,
    Environment,
}

pub trait ToEnv {
    fn to_env_path(&self) -> PathBuf;
}

impl ToEnv for Shortcut {
    fn to_env_path(&self) -> PathBuf {
        match self.kind {
            PathKind::Standard => {
                let parent = "$".to_owned() + &self.parent;
                PathBuf::from(parent).join(PathBuf::from(self.child.clone()))
            },
            PathKind::Environment => {
                PathBuf::from(self.parent.clone())
            },
        }
    }
}

pub fn to_bash(fp: &Path, shortcuts: Vec<Shortcut>) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(fp)?;
    file.write_all(b"#!/bin/bash\n\n")?;

    for s in shortcuts {
        let line = format!("export {}=\"{}\"\n", s.name.clone(), s.to_env_path().display());
        file.write_all(line.as_bytes())?;
    }
    Ok(())
}

pub fn get_home_dir() -> PathBuf {
    dirs::home_dir().expect("$HOME environment variable not initialized.")
}

pub fn get_path_variant(fp: &Path) -> PathKind {
    let starts_with = |base| fp.starts_with(base);

    if dirs::home_dir().is_some() && (
        starts_with("~")
        || starts_with("$HOME")
        || starts_with("${HOME}")) {
        return PathKind::Environment
    }
    PathKind::Standard
}

pub fn convert_parent_path(fp: &Path) -> String  {
    fp.parent().unwrap().file_name().unwrap().to_os_string().into_string().unwrap()
}

pub fn convert_child_path(fp: &Path) -> String {
    fp.file_name().unwrap().to_os_string().into_string().unwrap()
}

pub fn to_shortcuts(folders: Vec<walkdir::DirEntry>) -> Vec<Shortcut> {
    let shortcut_paths: Vec<Shortcut> = folders
        .into_iter()
        .map(|folder| {

            let fp = folder.into_path();
            let parent = fp.parent().unwrap().file_name().unwrap().to_os_string().into_string().unwrap();
            let child = convert_child_path(&fp);
            let name = child.clone().replace("-", "");

            Shortcut { name, parent, child, kind: PathKind::Standard }
    }).collect();
    shortcut_paths
}

// Root & SubstitutePrefix
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

    // Compact HOME Prefixes
    let root = Root { root };
    let (root, _) = compact_home(&root, "$HOME".to_string()).unwrap();

    let child = convert_child_path(&root);
    let root_shortcut = Shortcut {
        name: child.clone(),
        parent: root.display().to_string(),
        child,
        kind: PathKind::Environment
    };

    // Convert the children to standard shortcuts
    let mut shortcuts = vec![root_shortcut];
    shortcuts.append(&mut to_shortcuts(folders));
    
    println!("Created shortcuts for: ");
    shortcuts.iter().for_each(|s| println!("\t{}", s.to_env_path().display()));

    // Convert to bash script
    to_bash(&dest, shortcuts)?;
    println!("Wrote shortcuts to {}", dest.display());
    
    Ok(())
}
