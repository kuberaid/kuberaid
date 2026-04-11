#!/usr/bin/env -S just --justfile

set quiet := true
set shell := ['bash', '-euo', 'pipefail', '-c']

export KUBECONFIG := justfile_dir() + "/lab/kubeconfig"

mod lab "lab"
mod helm "helm"

[private]
default:
    just -l

[private]
log lvl msg *args:
    gum log -t rfc3339 -s -l "{{ lvl }}" "{{ msg }}" {{ args }}
