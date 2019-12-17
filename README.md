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
    -h, --help                      Prints help information
    -V, --version                   Prints version information

OPTIONS:
    -k, --bit-bucket-project-key <BitBucket Project key>
    -w, --bit-bucket-password <BitBucket password>
    -s, --server <BitBucket server base url, http://example.bitbucket.mycompany.com>
    -u, --bit-bucket-username <BitBucket user name>
    -t, --thread-count <Number of system threads>                                            [default: 3]
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

## Caveats
Update is not implemented, you'll need to do that with bash like this
```bash
# Alias to make it a bit simpler to handle
# Add it to your .bash_profile ?
alias git-pull-recursive='find . -maxdepth 3 -mindepth 2 -type d -name .git -exec sh -c "cd \"{}\"/../ && git reset --hard -q && git pull -q --ff-only &" \;'
```
