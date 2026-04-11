set positional-arguments := true
set quiet := true
set shell := ['bash', '-euo', 'pipefail', '-c']

helm_dir := justfile_dir() + '/helm'

crd_yaml := helm_dir + '/templates/crd.yaml'

vendor:
    just log "info" "Download dependency charts."
    helm repo add "bjw-s-labs" "https://bjw-s-labs.github.io/helm-charts"
    helm dependency build

crds:
    just log "info" "Generate crds."
    cargo run --bin crdgen > "{{ crd_yaml }}"

build: vendor crds
    just log "info" "Build helm chart."
    helm package "{{ helm_dir }}"

install: crds
    just log "info" "Apply helm chart to cluster."
    helm install kuberaid "{{ helm_dir }}"

upgrade: crds
    just log "info" "Apply helm chart to cluster."
    helm upgrade kuberaid "{{ helm_dir }}"

push:
    helm push "{{ helm_dir }}"
