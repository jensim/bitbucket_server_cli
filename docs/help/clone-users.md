```
bitbucket_server_cli-clone-users 0.3.13
Clone users

USAGE:
    bitbucket_server_cli clone-users [FLAGS] [OPTIONS]

FLAGS:
    -B, --batch           Run terminal in batch mode, with no interactions.
    -A, --all             Clone all projects
    -W, --env-password    Try get password from env variable BITBUCKET_PASSWORD.
                          Try it out without showing your password:
                          IFS= read -rs BITBUCKET_PASSWORD < /dev/tty  && export BITBUCKET_PASSWORD
    -H, --http-verbose    Output full http response on failed bitbucket requests.
    -Q, --git-quiet       Suppress warnings from failed git actions.
    -R, --reset           Reset repos before updating, and switch to master branch
    -h, --help            Prints help information
    -V, --version         Prints version information

OPTIONS:
        --http-backoff <backoff>
            Linear backoff time per failed request.
            ie. 10 timed out requests and backoff=10ms -> 100ms backoff on next timed out request
            or {prior_timeouts}*{backoff}={delay_on_next_timeout}
    -b, --concurrent-http <bitbucket_concurrency>
            Number of concurrent http requests towards bitbucket. Keep it sane, keep bitbucket alive for all. Max=100
            [default: 20]
    -w, --password <bitbucket_password>              BitBucket password
    -s, --server <bitbucket_server>                  BitBucket server base url, http://example.bitbucket.mycompany.com
    -u, --username <bitbucket_username>              BitBucket username
        --clone-type <clone_type>                     [default: ssh]  [possible values: SSH, HTTP, HttpSavedLogin]
    -g, --concurrent-git <git_concurrency>
            Number of concurrent git actions. Bitbucket might have a limited number of threads reserved for serving git
            requests - if you drive this value to high you might block your CI, colleagues or even crash bitbucket.
            Max=100 [default: 5]
    -k, --key <git_project_keys>...                  BitBucket Project keys (applicable multiple times)
        --output-directory <output-directory>        Suppress warnings from failed git actions. [default: .]
        --retries <retries>
            Retries to attempt requesting on timeout from bitbucket. [default: 2]

        --http-timeout <timeout>
            HTTP timout, 2min4sec6milli8micro3nano combine freely with or without abbreviations or spaces. [default: 2.5
            sec]

```
Use:
```
$> bitbucket_server_cli clone-users   
BitBucket server address: http://localhost
BitBucket username: jensim
✔ BitBucket password · ********
Clone/update all projects yes
Fetching users [00:00:15] [########################################] 1337/1337 (eta:0s)
Working repos [00:00:03] [########################################] 68/68 (eta:0s)
```
