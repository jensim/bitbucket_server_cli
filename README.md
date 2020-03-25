BitBucket Server Cli
----

[![Build Status](https://travis-ci.org/jensim/bitbucket_server_cli.svg?branch=master)](https://travis-ci.org/jensim/bitbucket_server_cli)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)


## Available options
```
BitBucket Server Cli 0.1.0

USAGE:
    bitbucket_server_cli [FLAGS] [OPTIONS] --server <BitBucket server base url, http://example.bitbucket.mycompany.com>

FLAGS:
    -W, --ask        Ask for password
    -A, --all        All project keys
    -V, --verbose    More verbose output
    -R, --reset      Reset repos before pulling/after cloning
    -h, --help       Prints help information
        --version    Prints version information

OPTIONS:
    -k, --key <BitBucket Project key>                                                   
    -w, --password <BitBucket password>                                                 
    -s, --server <BitBucket server base url, http://example.bitbucket.mycompany.com>    
    -u, --username <BitBucket user name>                                                
    -t, --threads <Number of system threads>                                             [default: 3]

```

## run from source
```bash
# Single project
cargo run -- -k KEY -s https://example.server.com
# All project found
cargo run -- -A -s https://example.server.com
# Skip checkout
cargo run -- -S -k KEY -s https://example.server.com
```

## install to run locally
```bash
cargo install --path . --force
bitbucket_server_cli -A -s https://example.server.com
```

## Caveats
- Update is currently implemented by delegating to sh git
- git auth expects the current shell user to have a valid id_rsa file in `$HOME/.ssh/`
- 
