#!/bin/sh
set -e

PRJ_ROOT=$(git rev-parse --show-toplevel)

cd "$PRJ_ROOT"
cargo install --path . --force

cd "${PRJ_ROOT}/integration_test/wiremock-bitbucket"
docker-compose up -d
until curl -f http://localhost:8080/some/thing 2>/dev/null; do
  echo >&2 "Wiremock is unavailable - sleeping"
  sleep 1
done

cd /tmp
rm -rf my_hot_repos
mkdir my_hot_repos
cd my_hot_repos
bitbucket_server_cli clone --all --batch --server=http://localhost:8080
test -f /tmp/my_hot_repos/active/linuxbrew-bitbucket_server_cli/Formula/bitbucket_server_cli.rb
test -f /tmp/my_hot_repos/\~jensim/homebrew-bitbucket_server_cli/Formula/bitbucket_server_cli.rb

cd "${PRJ_ROOT}/integration_test/wiremock-bitbucket"
docker-compose down

cd "${PRJ_ROOT}"

set +e
