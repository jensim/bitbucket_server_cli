Bitbucket server CLI
---

- [GitHub repo](https://github.com/jensim/bitbucket_server_cli/)
- [Usage](#usage)
- [Help](#help)
  - [clone projects](help/clone-projects.md)
  - [clone users](help/clone-users.md)
  - [generate completions](help/generate-completions.md)
- Distributions
  - [GitHub releases](https://github.com/jensim/bitbucket_server_cli/releases)
  - [![Homebrew](https://img.shields.io/badge/HomeBrew-repo-blue)](https://github.com/jensim/homebrew-bitbucket_server_cli/)
  - [![Linuxbrew](https://img.shields.io/badge/LinuxBrew-repo-red)](https://github.com/jensim/linuxbrew-bitbucket_server_cli-linux/)
- [Ideas/Plans](Ideas.md)

#### Usage

[![Screen recording](https://img.youtube.com/vi/9tVrG6uoUeM/0.jpg)](https://www.youtube.com/watch?v=9tVrG6uoUeM)

```
$> bitbucket_server_cli clone      
BitBucket server address: http://localhost
BitBucket username: jensim
✔ BitBucket password · ********
Clone/update all projects yes
Fetching users [00:00:15] [########################################] 2011/2011 (eta:0s)
Fetching projects [00:00:00] [########################################] 35/35 (eta:0s)
Working repos [00:01:07] [########################################] 1337/1337 (eta:0s)
```

#### Help

```
BitBucket Server Cli 0.3.13
Clone a thousand repos, and keep em up to date, no problem.

USAGE:
    bitbucket_server_cli <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    clone             Clone projects and users combined
    clone-projects    Clone projects
    clone-users       Clone users
    completions       Generate shell completions
    help              Prints this message or the help of the given subcommand(s)

```
