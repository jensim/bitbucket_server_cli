BitBucket Server Cli
----

[![GH-Build](https://github.com/jensim/bitbucket_server_cli/workflows/Rust/badge.svg?branch=main)](https://github.com/jensim/bitbucket_server_cli/actions?query=workflow%3ARust)
[![Rust audit](https://github.com/jensim/bitbucket_server_cli/workflows/Rust%20audit/badge.svg?branch=main)](https://github.com/jensim/bitbucket_server_cli/actions?query=workflow%3A%22Rust+audit%22)

[![repo dependency status](https://deps.rs/repo/github/jensim/bitbucket_server_cli/status.svg)](https://deps.rs/repo/github/jensim/bitbucket_server_cli)
[![release dependency status](https://deps.rs/crate/bitbucket_server_cli/0.3.15/status.svg)](https://deps.rs/crate/bitbucket_server_cli/0.3.15)

[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
![warnings forbidden](https://img.shields.io/badge/warnings-forbidden-success.svg)

![Crates.io](https://img.shields.io/crates/l/bitbucket_server_cli)
![Crates.io](https://img.shields.io/crates/v/bitbucket_server_cli)
![Crates.io](https://img.shields.io/crates/d/bitbucket_server_cli)
[![Homebrew](https://img.shields.io/badge/HomeBrew-repo-blue)](https://github.com/jensim/homebrew-bitbucket_server_cli/)
[![Linuxbrew](https://img.shields.io/badge/LinuxBrew-repo-red)](https://github.com/jensim/linuxbrew-bitbucket_server_cli-linux/)

![State](https://img.shields.io/badge/maintenance-working_but_experimental-blue.svg)
![Maintenance](https://img.shields.io/maintenance/yes/2021)

[![Screen recording](https://img.youtube.com/vi/9tVrG6uoUeM/0.jpg)](https://www.youtube.com/watch?v=9tVrG6uoUeM)

* [Install](#install)
  * [OSX](#osx)
  * [Linux](#linux)
  * [Windows](#windows)
* [Run](#run)
* [Disclaimer](#disclaimer)

## Install
#### OSX
```shell script
# From brew
brew install jensim/bitbucket_server_cli/bitbucket_server_cli

# From cargo
cargo install bitbucket_server_cli

# From source
cargo install --path . --force
```

#### Linux
& Windows subsystem Linux

https://github.com/jensim/linuxbrew-bitbucket_server_cli-linux/
```shell script
brew install jensim/bitbucket_server_cli-linux/bitbucket_server_cli
# or
brew tap jensim/linuxbrew-bitbucket_server_cli-linux git@github.com:jensim/linuxbrew-bitbucket_server_cli-linux.git
brew install bitbucket_server_cli
```

#### Windows
Head over to the [releases page](https://github.com/jensim/bitbucket_server_cli/releases) and snag the windows `*.exe` archive.
Or build from source. 
Or install from Cargo, which will build from source.

**Interactive mode doesn't work in `Cygwin`/`GitBash` terminals due to lacking support in dialoguer, stick to using `cmd.exe` and `PowerShell` terminals**

## Run
```shell script
# Fully interactive
bitbucket_server_cli clone

# Partially interactive
bitbucket_server_cli clone -s https://example.com

# Batch mode 
bitbucket_server_cli -B -s https://example.com -A

# 'Safe' password usage in batch mode. Depending on terminal, password might be seen in process description.
IFS= read -rs BITBUCKET_PASSWORD < /dev/tty
export BITBUCKET_PASSWORD
bitbucket_server_cli -s https://example.com -A -u jensim -W

# Run from source
cargo run -- clone
```

## git hooks
I've set up a little pre-commit bash-script that will run `fmt`, `clippy` & `integration-tests`
````shell script
git config core.hooksPath .githooks
# or
./.githooks/pre-commit
````

## Disclaimer
- Only tested on Mac OS X
- Use at own risk
- You are responsible for any and all actions you perform with this tool
  - Legal
  - Company policy
  - Any other
