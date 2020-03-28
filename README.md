BitBucket Server Cli
----

[![Build Status](https://travis-ci.org/jensim/bitbucket_server_cli.svg?branch=master)](https://travis-ci.org/jensim/bitbucket_server_cli)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

## install
```
cargo install bitbucket_server_cli
```

## run from source
```bash
➜  bitbucket_server_cli git:(master) ✗ cargo run
BitBucket server address: https://example.com
BitBucket username: user
BitBucket password: [hidden]
Clone/update all found projects with repos yes
Thread count: 3
Reset state no
Verbose output yes
Started working 1337 repositories
[00:01:41] [########################################] 1337/1337 (eta:0s)
Done in 1 minute
➜  bitbucket_server_cli git:(master) ✗ 
```

## install from source
```bash
cargo install --path . --force
bitbucket_server_cli
```

## Caveats
- Update is currently implemented by delegating to shell git
- git auth expects the current shell user to have a valid id_rsa file in `$HOME/.ssh/`
- only tested on Mac OS X
