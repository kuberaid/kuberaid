set positional-arguments := true
set quiet := true
set shell := ['bash', '-euo', 'pipefail', '-c']

CONTAINER := "podman"
REGISTRY_PORT := "5000"
REGISTRY_ADDR := "localhost:" + REGISTRY_PORT

image_dir := justfile_dir() + '/image'

build name:
    just log "info" "Building image {{name}}"
    {{CONTAINER}} build -t "{{REGISTRY_ADDR}}/kuberaid-{{name}}:dev" -f {{ image_dir + "/" + name + ".Containerfile" }} {{ justfile_dir() }}

push name: (build name)
    just log "info" "Pushing image {{name}}"
    {{CONTAINER}} push "{{REGISTRY_ADDR}}/kuberaid-{{name}}:dev" --tls-verify=false
