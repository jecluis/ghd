#!/bin/bash

if ! graphql-client --help >&/dev/null; then
  echo "error: missing graphql-client; please install with cargo." >/dev/stderr
  echo "  >> cargo install graphql_client_cli" >/dev/stderr
  exit 1
fi

if [[ ! -e "gh.schema.graphql" ]]; then
  echo "error: must be run from src/gh/gql/" >/dev/stderr
  exit 1
fi

graphql-client generate \
  --custom-scalars-module 'crate::gh::gql::custom_types' \
  --response-derives 'Debug' \
  --schema-path gh.schema.graphql \
  queries.graphql
