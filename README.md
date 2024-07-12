# relconf

[![standard-readme compliant](https://img.shields.io/badge/standard--readme-OK-green.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)

Config relative to the current path

`relconf` generates toml configuration files based on the current path. It reads its configuration from a yaml file
(see [Usage](#usage) for where that file is expected to be), merges toml files based on the current path, writes the
final configuration to disk and optionally outputs an enivorment variable pointing to the generated config.

## Table of Contents

- [Install](#install)
- [Usage](#usage)
- [Maintainers](#maintainers)
- [Contributing](#contributing)
- [License](#license)

## Install

Clone the repo before running the following command and make sure you have a recent version of the
[Rust toolchain](https://rustup.rs/) installed.

```sh
cargo install --path .
```

## Usage

```sh
# generate configs for all tools
relconf
# generate configs only for listed tools
relconf -o foo,bar
```

## `relconf` in practice

Relconf will not run automatically. I recommend writing a wrapper around tools that use `relconf` to manage their
configuration like so and adding the wrapper to your `.bashrc`/`.zshrc`/`config.fish` (or similar).

- Bash/ZSH:

  ```sh
  # as an alias
  alias foo="source <(relconf -o foo); command foo"
  # or as a function
  function foo() {
    source <(relconf -o foo)
    command foo $@
  }
  ```

- Fish:

  ```sh
  function foo --wraps foo
    relconf -o foo | source
    command foo $argv
  end
  ```

Set up like this `relconf` will always run before the command `foo` gets executed, ensuring the configuration is
current and appropriate for whatever directory you're running `foo` in.

## `relconf`'s config file

`relconf` reads its configuration from disk. The following is an example configuration that uses all features.

```yaml
tools:
- name: jj
  format: toml # right now only toml is supported
  rootconfig: ~/.jjconfig.toml # mandatory
  inject:
    - env-name: JJ_CONFIG # optional
      path: ~/.config/jj/merged.toml
    - path: ~/.config/jj/other-location.toml
  subconfigs:
    - config: ~/.config/jj/always.toml
    - config: ~/.config/jj/company.toml
      when: # optional, when absent the subconfig will always be imported
        - directory: ~/workspace/company-gitlab
          match-subdirectories: true # optional, defaults to false
```

`relconf` preserves the order in which subconfigs are listed and will merge and overwrite values in that order as well.

If an injection has the `env-name` key set `relconf` will output something like this (based on the above config):

```sh
export JJ_CONFIG=/home/USERNAM/.config/jj/merged.toml
```

The output to stdout made by `relconf` is safe to `source` in Bash, ZSH (and other posix compatible shells) and Fish.
This makes it easy to automatically set the approriate configuration variable like the snippets in the
[section above](#relconf-in-practice) do.

There's also a JSON Schmema for the `relconf` config format available in [/assets/relconf.schema.json].

### Config file location

The default location for the config file depends on your operating system.
See the table below for an overview. Note that only Linux, macOS and Windows are supported.

| OS      | default config path                                                           |
|---------|-------------------------------------------------------------------------------|
| Linux   | `$XDG_CONFIG_HOME/relconf/config.yaml` or `$HOME/.config/relconf/config.yaml` |
| macOS   | `$HOME/Library/Application Support/relconf/config.yaml`                       |
| Windows | `C:\Users\USERNAME\AppData\Roaming\relconf\config.yaml`                       |

You can override the default location by setting `RELCONF_CONFIG` or by using `relconf -c path/to/config.yaml`.

### generting the schema

To generate the schema, `relconf` needs to be built with the `schmema` feature enabled. You can enable the feature and
generate the schema like so:

```sh
cargo run -F schema -- --generate-schema
```

## Maintainers

[@kfkonrad](https://github.com/kfkonrad)

## Contributing

PRs accepted.

Small note: If editing the README, please conform to the
[standard-readme](https://github.com/RichardLitt/standard-readme) specification.

## License

MIT © 2024 Kevin F. Konrad
