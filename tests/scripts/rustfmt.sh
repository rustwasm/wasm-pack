#!/bin/bash

which rustfmt > /dev/null

if [[ $? -ne 0 ]]; then
  cargo install rustfmt
else
  current_version=$(rustfmt --version | cut -d '-' -f 1 | xargs)
  upstream_version=$(cargo search rustfmt | head -n 1 | cut -d ' ' -f 3 | tr -d '"' | xargs)
  if [ $current_version != $upstream_version ]; then
    cargo install rustfmt --force
  fi
fi
