use std::{fs, path::{Path, PathBuf}};

use toml::Table;
use shellexpand::path::tilde;

pub fn normalize<SP>(path: &SP) -> PathBuf
where
    SP: ?Sized + AsRef<Path>,
{
  tilde(path).canonicalize().unwrap()
}

pub fn is_subdir(parent: &PathBuf, subdir: &PathBuf) -> bool {
  let parent_canon = normalize(parent);
  let subdir_canon = normalize(subdir);

  subdir_canon.starts_with(parent_canon)
}

pub fn is_file(path: &Path) -> bool {
  normalize(&path).is_file()
}

pub fn is_dir(path: &Path) -> bool {
  normalize(&path).is_dir()
}

pub fn read_toml(path: &PathBuf) -> Table {
  std::fs::read_to_string(normalize(path)).unwrap().parse().unwrap()
}

pub fn write_toml(path: &PathBuf, table: &Table) {
  let path = normalize(path);
  let parent = path.parent().unwrap();
  fs::create_dir_all(parent).unwrap();
  fs::write(path, table.to_string()).unwrap();
}
