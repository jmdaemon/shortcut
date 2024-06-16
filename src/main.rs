use std::{
    error::Error, 
    fs::File,
    io::Write,
    path::{Path, PathBuf}
};

use clap::Parser;

use shortcut::app::Args;
use walkdir::WalkDir;

// Treat roots differently from any other kind of directory
//pub struct ShortcutRootPath {
    //pub root: String,
//}

pub struct Shortcut {
    pub name: String,
    pub path: ShortcutPath,
}

//pub struct ShortcutsTree {
    //pub root: ShortcutRootPath,
    //pub branches: Vec<ShortcutPath>
//}

//pub struct Shortcut {
    //pub name: String,
    //pub path: ShortcutPath,
//}

#[derive(Clone, Debug)]
//pub enum ShortPathKind {
pub enum PathKind {
    Standard,
    Environment,
}

//pub enum PathKind {
    //Child,
    //Root,
//}

#[derive(Clone)]
//pub struct ShortPath {
pub struct PathVariant {
    pub path: String,
    pub kind: PathKind,
}

pub struct ShortcutPath {
    pub parent: PathVariant,
    pub child: String,
}

pub trait ToEnv {
    //fn to_env_string(&self) -> String;
    fn to_env_path(&self) -> PathBuf;
}

impl ToEnv for ShortcutPath {
    /*
    fn to_env_string(&self) -> String {
        //let path_variant = self.parent.to_owned();
        //let (path, kind) = (path_variant.path,;
        
        //let kind =  self.parent.kind;

        let to_shortcut_path = |parent, child| {
            "$".to_owned() + parent;
            PathBuf::from(parent).join(PathBuf::from(child))
        };

        //let parent = match self.parent.kind {
        let result = match self.parent.kind {
            PathKind::Standard => {
                //"$".to_owned() + &self.parent.path
                to_shortcut_path(&self.parent.path, self.child.clone())
            },
            PathKind::Environment => {
                //get_home_dir().into_os_string().into_string().unwrap()
                get_home_dir()
            }
        };
        result

        //let parent = match self.parent {
            //PathVariant { path, kind } => {
            //}
            //PathKind::Environment {}
        //}

        //let parent = "$".to_owned() + &self.parent;
        //let result = PathBuf::from(parent).join(PathBuf::from(self.child.clone()));
        //result.into_os_string().into_string().unwrap()
        
    }
    */

    fn to_env_path(&self) -> PathBuf {
        match self.parent.kind {
            PathKind::Standard => {
                let parent = "$".to_owned() + &self.parent.path;
                PathBuf::from(parent).join(PathBuf::from(self.child.clone()))
            },
            //PathKind::Environment => get_home_dir()
            //PathKind::Environment => PathBuf::from(self.parent.path.clone()),
            PathKind::Environment => {
                PathBuf::from(self.parent.path.clone())
                //PathBuf::from("~").join(self.parent.path.clone())
            },
        }
        //let parent = "$".to_owned() + &self.parent;
        //let result = PathBuf::from(parent);
        //result.join(PathBuf::from(self.child.clone()))
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
    //if let Some(home) = dirs::home_dir() {
        //if fp.starts_with("~") || fp.starts_with("$HOME") {
            //return PathKind::Environment
        //}
    //}
    //PathKind::Standard

    if dirs::home_dir().is_some() 
        && (fp.starts_with("~")
        || fp.starts_with("$HOME")
        || fp.starts_with("${HOME}")) {
            return PathKind::Environment
    }
    PathKind::Standard
}

pub fn convert_path(fp: &Path, path_kind: PathKind) -> PathVariant {
    let path = match path_kind {
        PathKind::Standard => { fp.parent().unwrap().file_name().unwrap().to_os_string().into_string().unwrap() },
        PathKind::Environment => {
            //let home = get_home_dir();
            //home.into_os_string().into_string().unwrap()
            "~".to_owned()
        }
    };
    PathVariant { path, kind: path_kind }
}

pub fn to_shortcut_paths(folders: Vec<walkdir::DirEntry>) -> Vec<ShortcutPath> {
    let shortcut_paths: Vec<ShortcutPath> = folders
        .into_iter()
        .map(|folder| {
            let fp = folder.into_path();


            // What do we want?
            // We want control over how our path gets converted into a shortcut path

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

            let path_variant = convert_path(&fp, path_kind);
            //println!("Path Variant: {:?}", path_kind);
            let child = fp.file_name().unwrap().to_os_string().into_string().unwrap();
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

pub fn starts_with_home_prefix(fp: &Path) -> bool {
    fp.starts_with("~")
        || fp.starts_with("$HOME")
        || fp.starts_with("${HOME}")
}

pub enum HomePathVarKind {
    TILDE,
    HOME,
    HOME_CURLY,
}

pub struct HomePathVarVariant {
    pub base: String,
    pub kind: HomePathVarKind,
}

pub fn get_home_path_variant() {
}


// Root & ExpandPrefix
#[derive(Clone)]
pub struct Root {
    pub root: PathBuf,
}

pub trait ExpandPrefix {
    fn starts_with(&self, base: &str) -> bool;
    fn expand_prefix(&self, base: &str, expands_to: String) -> Result<(PathBuf, String), Box<dyn Error>>;
}

impl ExpandPrefix for Root {
    fn starts_with(&self, base: &str) -> bool {
        self.root.starts_with(base)
    }

    fn expand_prefix(&self, base: &str, expands_to: String) -> Result<(PathBuf, String), Box<dyn Error>> {
        if self.starts_with(base) {
            let children = self.root.strip_prefix(base)?;
            let prefix = PathBuf::from(expands_to);
            let root = prefix.join(children);
            return Ok((root, base.to_string()))
        }
        panic!("Could not expand prefix");
    }
}

//pub fn expand_prefix(root: &Root, base: &str) -> Option<PathBuf> {
pub fn expand_prefix(root: &Root, base: &str) -> Option<(PathBuf, String)> {
    if root.starts_with(base) {
        let root_span = root
            .expand_prefix(base, get_home_dir().display().to_string())
            .unwrap_or_else(|_| panic!("Could not expand prefix for {}", root.root.display()));
        //println!("{:?}", root_span.clone());
            //.expect(&format!("Could not expand prefix for {}", root.root.display()));
        return Some(root_span);
    }
    None
}

//pub fn expand_home(root: &Root) -> Option<PathBuf> {
pub fn expand_home(root: &Root) -> Option<(PathBuf, String)> {
    //let expand = |base| expand_prefix(root, base);
    let expand = |base| expand_prefix(root, base);
    //expand_prefix(root, "~")
        //.map_or(None , |_| expand_prefix(root, "$HOME"))
        //.map_or(None , |_| expand_prefix(root, "${HOME}"))
    //expand_prefix(root, "~")
        //.and_then(|_| expand_prefix(root, "$HOME"))
        //.and_then(|_| expand_prefix(root, "${HOME}"))

    //expand("~")
        //.map_or_else(|| expand("$HOME"), expand("${HOME}"))

        //.and_then(|_| expand("$HOME"))
        //.and_then(|_| expand("${HOME}"))
        //.and_then(|_| expand_prefix(root, "~"))

    //root.expand_prefix("~", get_home_dir())

    //expand_prefix(root, "~")
        //.map_or_else(|| expand_prefix(root, "$HOME"), Some)
        //.map_or_else(|| expand_prefix(root, "${HOME}"), Some)

    expand("~")
        .map_or_else(|| expand("$HOME"), Some)
        .map_or_else(|| expand("${HOME}"), Some)

}

pub trait ContractPrefix {
    fn sub_for(&self, base: &str) -> bool;
    fn contract_prefix(&self, base: &str, expands_to: String) -> Result<(PathBuf, String), Box<dyn Error>>;
}

impl ContractPrefix for Root {
    fn sub_for(&self, base: &str) -> bool {
        self.root.starts_with(base)
    }

    fn contract_prefix(&self, base: &str, contracts_to: String) -> Result<(PathBuf, String), Box<dyn Error>> {
        if self.sub_for(base) {
            let children = self.root.strip_prefix(base)?;
            let prefix = PathBuf::from(contracts_to);
            let root = prefix.join(children);
            return Ok((root, base.to_string()))
        }
        panic!("Could not contract prefix");
    }
}

pub fn contract_prefix(root: &Root, base: &str) -> Option<(PathBuf, String)> {
    if root.starts_with(base) {
        let root_cont = root
            .contract_prefix(base, get_home_dir().display().to_string())
            .unwrap_or_else(|_| panic!("Could not expand prefix for {}", root.root.display()));
        return Some(root_cont);
    }
    None
}

//pub fn span_path_exists(sp: Option<PathBuf>) -> bool {
pub fn span_path_exists(sp: Option<(PathBuf, String)>) -> bool {
    //sp.is_none() || sp.map_or(false, |f| !f.exists())
    //sp.is_none() || sp.map_or_else(false, |f| !f.exists())
    //sp.is_none() || sp.map(|f| !f.exists()).unwrap()
    sp.is_none() || sp.unwrap().0.exists()
}


fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    
    let (root, depth, dest) = (args.root, args.depth, args.dest);

    // Expand root
    let root = Root { root };

    //let expand_prefix = |base| { expand_home(root, base) };
    let span_root = expand_home(&root);

    //let (span_root_path, prefix) = span_root;
    //println!("{:?}", span_root.clone());

    if !span_path_exists(span_root.clone()) {
    //if !span_path_exists(span_root_path.clone()) {
        eprintln!("{} does not exist.", root.root.display());
        panic!("Root folder does not exist.");
    } 
    //let root = span_root.unwrap();
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


    // Root
    //let child = root.clone();

    let root = Root { root };
    let child = root.root.clone();

    // Change to comp_root
    // Change to shrink
    //let root_obj = Root { root };
    //let cont_root = root.contract_prefix("~", get_home_dir().display().to_string());
    let cont_root = root.contract_prefix(&get_home_dir().display().to_string(), "~".to_string());

    //let contract = |base| contract_prefix(&root, base);

    //let contract = |base| contract_prefix(&cont_root, base);
    //let cont_root = contract(get_home_dir().to_str().unwrap());
    let (comp_root, prefix) = cont_root.unwrap();

    //if !cont_path_exists(cont_root.clone()) {

    //if !span_path_exists(cont_root.clone()) {
        ////if !span_path_exists(cont_root.clone()) {
        //eprintln!("{} does not exist.", root.root.display());
        //panic!("Prefix does not exist.");
    //}

    //let (comp_root, prefix) = cont_root.unwrap();

    let root_shortcut = ShortcutPath { 
        parent: PathVariant {
            //path: root.display().to_string(),
            path: comp_root.display().to_string(),
            kind: PathKind::Environment
        },
        //child: root.file_name().unwrap().to_os_string().into_string().unwrap()
        child: child.file_name().unwrap().to_os_string().into_string().unwrap()
    };

    let mut shortcut_paths = to_shortcut_paths(folders);
    
    shortcut_paths.reverse();
    shortcut_paths.push(root_shortcut);
    shortcut_paths.reverse();

    let shortcuts = to_shortcuts(shortcut_paths);
    
    println!("Creating shortcuts for: ");
    //shortcuts.iter().for_each(|s| println!("\t{}", s.path.to_env_string()));
    shortcuts.iter().for_each(|s| println!("\t{}", s.path.to_env_path().display()));

    // Convert to bash script
    to_bash(&dest, shortcuts)?;
    println!("Wrote shortcuts to {}", dest.display());
    
    Ok(())

    //if let Some(root) = span_root {
        //if !root.exists() {
            //eprintln!("{} does not exist.", root.display());
            //panic!("Root folder does not exist.");
        //}
    //} else {
        //eprintln!("{} does not exist.", root.root.display());
        //panic!("Root folder does not exist.");
    //}
    


    //if span_root.is_none() || ( )

    //let span_root = expand_prefix("~").map.or(expand_prefix("$HOME"));
    //let span_root = expand_prefix("$HOME").map_or(|a, b| { expand_prefix("$HOME")});
    //let span_root = expand_home(root, "~")
        //.or_else(|f| {

        //});
//|f| expand_home(root, "$HOME")
        //.or_else(expand_home(root, "${HOME}"));

    //if root.starts_with("~") {
        //let root_span =
            //root.expand_prefix("~", get_home_dir().display().to_string())
            //.expect(&format!("Could not expand prefix for {}", root.root.display()));
    //}


    //let mut root = args.root;
    //let mut original = root.clone();

    // Common HOME variants
    //let home_tilde = HomePathVarVariant { base: "~".to_string(), kind: HomePathVarKind::TILDE };
    //let home = HomePathVarVariant { base: "$HOME".to_string(), kind: HomePathVarKind::HOME };
    //let home_curly = HomePathVarVariant { base: "${HOME}".to_string(), kind: HomePathVarKind::HOME_CURLY };

    // Does root start with home?
    //if root.starts_with(&home_tilde.base) {
        //let rest = root.strip_prefix(home_tilde.base)?;
        //root = get_home_dir().join(rest);
        //println!("Starts with tilde");
        //println!("{}", root.display())
    //} else if root.starts_with(&home.base) {
        //let rest = root.strip_prefix(home.base)?;
        //root = get_home_dir().join(rest);
    //} else if root.starts_with(&home_curly.base) {
        //let rest = root.strip_prefix(home_curly.base)?;
        //root = get_home_dir().join(rest);
    //}

    // The issue comes when we treat root as a homogenous directory
    // We both want to treat root homogenously and yet differently

    // Solution:
    // We want our program to generate all the shortcuts including the one for our root.
    // However, we don't want the root to be generated the same as the other files
    // Therefore we'll treat the root directory completely separately, and add the rest back in later

    //let root = args.root.display().to_string().replace(, to)
    //let prefix = args.root.strip_prefix(
    //let root =  .display().to_string().replace(, to)

    // Does root start with home?

    // Expand
    //if starts_with_home_prefix(root) {
        //let rest = args.root.strip_prefix(
        //root = get_home_dir().join(path)
    //}


    //let depth = args.depth;
    //let dest = args.dest;

    //if !root.exists() {
        //eprintln!("{} does not exist.", root.display());
        //panic!("Root folder does not exist.");
    //}
    


    // Remove root and treat it separately

    
}
