BitBucket Server Cli
----

[![Build Status](https://travis-ci.org/jensim/bitbucket_server_cli.svg?branch=master)](https://travis-ci.org/jensim/bitbucket_server_cli)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
![Crates.io](https://img.shields.io/crates/l/bitbucket_server_cli)
![Crates.io](https://img.shields.io/crates/v/bitbucket_server_cli)
![Crates.io](https://img.shields.io/crates/d/bitbucket_server_cli)
![Maintenance](https://img.shields.io/badge/maintenance-experimental-blue.svg)

* [Install](#install)
* [Run](#run)
* [Caveats](#caveats)

## Install
```shell script
# From brew
brew install jensim/bitbucket_server_cli/bitbucket_server_cli

# From cargo
cargo install bitbucket_server_cli

# From source
cargo install --path . --force
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
bitbucket_server_cli -s https://example.com -A -u jensim -w $BITBUCKET_PASSWORD

# Run from source
cargo run -- -I
```

## Caveats
- only tested on Mac OS X
