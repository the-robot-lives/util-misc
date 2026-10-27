# Project Schema — misc-git-utils
<!-- 〚📁:41252e5d-a399-54cb-b4ae-ae37c1a0d0a5〛 Project Schema :: No persistence layer; scripts write through git only. -->


This package has no database, configuration schema, or persistent data. Its four Bash commands operate on Git repositories and write through Git only when explicitly invoked. The only file-shaped artifact the package manages is the set of scripts it installs into `~/.local/bin` (`make install`); there is no config file, KV store, fixture corpus, or network data interface.

This repository carries its own `.meta/pointers.yaml` doc-pointer store; the record format and tooling are owned by [doc-pointers](https://github.com/the-robot-lives/doc-pointers).
