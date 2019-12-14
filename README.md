BitBucket Server Cli
----

[![Build Status](https://travis-ci.org/jensim/bitbucket_server_cli.svg?branch=master)](https://travis-ci.org/jensim/bitbucket_server_cli)

## Available options
```
BitBucket Server Cli 0.1.0

USAGE:
    bitbucket_server_cli [FLAGS] [OPTIONS] --server <BitBucket server base url, http://example.bitbucket.mycompany.com>

FLAGS:
    -A, --bit_bucket_project_all    
    -S, --skip-checkout             
    -h, --help                      Prints help information
    -V, --version                   Prints version information

OPTIONS:
    -k, --bit-bucket-project-key <BitBucket Project key>                                    
    -w, --bit-bucket-password <BitBucket password>                                          
    -s, --server <BitBucket server base url, http://example.bitbucket.mycompany.com>        
    -u, --bit-bucket-username <BitBucket user name>                                         
    -p, --git-ssh-password <SSH private key password to auth against BitBucket git repo>
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
```

### Caveats / TODOS
- git update only supported by passing `sh -C $PATH git update` to the OS. 
