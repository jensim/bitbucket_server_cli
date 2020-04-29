## Cargo
```bash
# Bump version in Cargo.toml
cargo login
cargo publish
```
## Brew
[Good instructions!!](https://federicoterzi.com/blog/how-to-publish-your-rust-project-on-homebrew/)
* Travis will draft a release for us
  * https://travis-ci.org/github/jensim/bitbucket_server_cli
    * Checksum can be found in the end of the log for each of the builds
      * Linux
      * OSX
  * Once successfull hit the publish button in GitHub GUI
    * https://github.com/jensim/bitbucket_server_cli/releases
* Clone https://github.com/jensim/homebrew-bitbucket_server_cli
  * Edit Formula/bitbucket_server_cli.rb (edit placeholders within <>)
  ```rb
  url "https://github.com/jensim/bitbucket_server_cli/releases/download/<VERSION>/bitbucket_server_cli-osx.tar.gz"
  sha256 "<SHA_FROM_BEFORE>"
  version "<VERSION>"
  ```
  * Commit 
    * `git add Formula/bitbucket_server_cli.rb` 
    * `git commit -m "Version <VERSION> release"`
  * Add a git tag
    * `git tag -a <VERSION> -m "<VERSION>"`
  * Push
    * `git push origin master`
  * Brew update
    * `brew update`
    * `brew info jensim/bitbucket_server_cli/bitbucket_server_cli`
* Clone https://github.com/jensim/linuxbrew-bitbucket_server_cli
  * Same as the homebrew one
