## Cargo
```bash
# Bump version in Cargo.toml
cargo login
cargo publish
```
## Brew
[Good instructions!!](https://federicoterzi.com/blog/how-to-publish-your-rust-project-on-homebrew/)
* GitHub Actions is set up to draft a release for us
  * https://github.com/jensim/bitbucket_server_cli/blob/main/.github/workflows/rust.yml
    * Checksum can be found in the release draft bodies  
  * Once successful, hit the publish button in GitHub GUI
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
    * `git push origin main`
  * Brew update
    * `brew update`
    * `brew info jensim/bitbucket_server_cli/bitbucket_server_cli`
* Clone https://github.com/jensim/linuxbrew-bitbucket_server_cli
  * Same as the homebrew one
