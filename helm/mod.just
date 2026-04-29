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
    cargo run --bin crdgen --features=crdgen > "{{ crd_yaml }}"

build: vendor crds
    just log "info" "Build helm chart."
    helm package "{{ helm_dir }}"

ns:
    just log "info" "Create namespace."
    kubectl create ns kuberaid-system &>/dev/null || true
    kubectl label ns kuberaid-system pod-security.kubernetes.io/enforce=privileged

apply *args: crds ns
    just log "info" "Apply helm chart to cluster."
    helm upgrade -i -n kuberaid-system kuberaid "{{ helm_dir }}" {{ args }}

push:
    helm push "{{ helm_dir }}"
