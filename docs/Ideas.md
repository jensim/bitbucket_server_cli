Ideas:
----
-[ ] Disaster recovery
  - Set up all projects
  - Set up all repos
  - Push all history 
-[ ] Set up integration-test bitbucket server
-[ ] `Prune` subcommand, remove directories not found in bitbucket
-[ ] Separate `Structopt`-structs from valid domain structs and contain that logic
-[ ] Set other defaults with `~/.config/bitbucket_server_cli/defaults`
-[ ] Mega-PR subcommand
-[ ] Homebrew run `generate completions` `post flight`
-[ ] Windows installer
  - Set `PATH` variable after install
  `setx "%path%;C:\Program Files\bitbucket_server_cli\bin”`
  - Download latest release from github, reusable installer

Done:
----
-[x] Clone project repos
-[x] Clone user repos
-[x] Clone combined
-[x] CLI completion
-[x] Timeouts
-[x] Retry on timeout
-[x] Backoff on timeout
