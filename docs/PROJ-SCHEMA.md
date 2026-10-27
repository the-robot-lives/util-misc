# Project Schema — misc-git-utils

This package has no database, configuration schema, or persistent data. Its four Bash commands operate on Git repositories and write through Git only when explicitly invoked. The only file-shaped artifact the package manages is the set of scripts it installs into `~/.local/bin` (`make install`); there is no config file, KV store, fixture corpus, or network data interface.

Doc-pointer records and their `.meta/pointers.yaml` format are owned by [doc-pointers](https://github.com/the-robot-lives/doc-pointers).
