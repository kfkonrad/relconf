use std::{fs, path::{Path, PathBuf}};

use dirs::home_dir;
use toml::Table;

fn expand_tilde(path: &PathBuf) -> PathBuf {
  if path.starts_with("~") {
      if let Some(home_dir) = home_dir() {
          let mut path_without_tilde = path.components();
          path_without_tilde.next();
          let path_without_tilde: PathBuf = path_without_tilde.collect();
          return home_dir.join(path_without_tilde);
      }
  }
  path.clone()
}

pub fn is_subdir(parent: &PathBuf, subdir: &PathBuf) -> bool {
  let parent_canon = fs::canonicalize(&expand_tilde(parent)).unwrap();
  let subdir_canon = fs::canonicalize(&expand_tilde(subdir)).unwrap();

  subdir_canon.starts_with(parent_canon)
}

pub fn is_file(path: &Path) -> bool {
  expand_tilde(&path.to_path_buf()).is_file()
}

pub fn is_dir(path: &Path) -> bool {
  expand_tilde(&path.to_path_buf()).is_dir()
}

pub fn read_toml(path: &PathBuf) -> Table {
  std::fs::read_to_string(expand_tilde(path)).unwrap().parse().unwrap()
}

pub fn write_toml(path: &PathBuf, table: &Table) {
  let path = expand_tilde(path);
  let parent = path.parent().unwrap();
  fs::create_dir_all(parent).unwrap();
  fs::write(path, table.to_string()).unwrap();
}
