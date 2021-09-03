#!/bin/sh
set -e

PRJ_ROOT=$(git rev-parse --show-toplevel)

cd "$PRJ_ROOT"
cargo install --path . --force

cd "${PRJ_ROOT}/integration_test/wiremock-bitbucket"
docker-compose up -d
wait_until="$(( $(date +%s) + 60 ))"
until curl -s -f http://localhost:8080/some/thing > /dev/null 2>&1 ; do
  if [ "$(date +%s)" -gt "$wait_until" ] ; then
    echo >&2 "Wiremock is unavailable - timed out"
    exit 1
  fi
  echo >&2 "Wiremock is unavailable - sleeping"
  sleep 1
done
echo >&2 "Wiremock is available - testing"

cd /tmp
rm -rf my_hot_repos
mkdir my_hot_repos
cd my_hot_repos
bitbucket_server_cli clone --all --batch --clone-type=http --server=http://localhost:8080/
test -f /tmp/my_hot_repos/active/linuxbrew-bitbucket_server_cli/Formula/bitbucket_server_cli.rb
test -f /tmp/my_hot_repos/\~jensim/homebrew-bitbucket_server_cli/Formula/bitbucket_server_cli.rb

cd "${PRJ_ROOT}/integration_test/wiremock-bitbucket"
docker-compose down

cd "${PRJ_ROOT}"

set +e
echo 'Great success!'
