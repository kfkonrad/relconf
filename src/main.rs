use tool::handle;

mod conf;
mod tool;
mod path;

fn main() {
    // TODO: read from config file
    // TODO: add CLI with option to override config file
    let yaml_data = r#"
tools:
- name: jj
  format: toml
  rootconfig: ~/.config/jj/config.toml
  inject:
    - type: env
      name: JJCONFIG
      location: ~/.config/relconf/JJCONFIG
    - type: file
      location: ~/.config/relconf/JJCONFIG2
  subconfigs:
    - directory: ~/workspace
      config: ~/.config/jj/workspace.toml
      match-subdirectories: true
    - directory: ~/workspace/github/kfkonrad
      config: ~/.config/jj/kfkonrad.toml
      match-subdirectories: true
"#;
    let config: conf::RelConf = serde_yaml::from_str(yaml_data).unwrap(); // TODO: remove unwraps (later)
    config.tools.into_iter().for_each(|tool| {
        handle(tool)
    });
}
