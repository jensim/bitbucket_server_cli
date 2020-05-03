BitBucket Server Cli
----

![GH-Build](https://github.com/jensim/bitbucket_server_cli/workflows/Rust/badge.svg?branch=master)
[![Travis-Build](https://travis-ci.org/jensim/bitbucket_server_cli.svg?branch=master)](https://travis-ci.org/jensim/bitbucket_server_cli)

[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
![Crates.io](https://img.shields.io/crates/l/bitbucket_server_cli)
![Crates.io](https://img.shields.io/crates/v/bitbucket_server_cli)
![Crates.io](https://img.shields.io/crates/d/bitbucket_server_cli)
![State](https://img.shields.io/badge/maintenance-working_but_experimental-blue.svg)
![Maintenance](https://img.shields.io/maintenance/yes/2020)

* [Install](#install)
  * [OSX](#osx)
  * [Linux](#linux)
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

## Run
```shell script
# Fully interactive
bitbucket_server_cli -I

# Partially interactive
bitbucket_server_cli -I -s https://example.com

# Batch mode 
bitbucket_server_cli -s https://example.com -A

# 'Safe' password usage in batch mode. Depending on terminal, password might be seen in process description.
IFS= read -rs BITBUCKET_PASSWORD < /dev/tty
export BITBUCKET_PASSWORD
bitbucket_server_cli -s https://example.com -A -u jensim -w $BITBUCKET_PASSWORD

# Run from source
cargo run -- -I
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
  - Leagal
  - Company policy
  - Any other
