## Cargo
```bash
# Bump version in Cargo.toml
cargo login
cargo publish
```
## Brew
[Good instructions!!](https://federicoterzi.com/blog/how-to-publish-your-rust-project-on-homebrew/)
* Package the binary for upload:
```bash
rm -rf target/release
cargo build --release
cd target/release
tar -czf bitbucket_server_cli-mac.tar.gz bitbucket_server_cli
shasum -a 256 bitbucket_server_cli-mac.tar.gz
```
* [Draft new github release](https://github.com/jensim/bitbucket_server_cli/releases/new)
  * Upload the archive bitbucket_server_cli-mac.tar.gz to the release
* Clone https://github.com/jensim/homebrew-bitbucket_server_cli
  * Edit Formula/bitbucket_server_cli.rb (edit placeholders within <>)
  ```rb
  url "https://github.com/jensim/bitbucket_server_cli/releases/download/<VERSION>/bitbucket_server_cli-mac.tar.gz"
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
