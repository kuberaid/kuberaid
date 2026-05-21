set positional-arguments := true
set quiet := true
set shell := ['bash', '-euo', 'pipefail', '-c']

lab_dir := justfile_dir() + '/lab'
gen_dir := lab_dir + '/gen'

REGISTRY_TLS_DIR := gen_dir + "/registry-tls"
REGISTRY_CERT := REGISTRY_TLS_DIR + "/domain.crt"
REGISTRY_KEY := REGISTRY_TLS_DIR + "/domain.key"
REGISTRY_CA_KEY := REGISTRY_TLS_DIR + "/ca.key"
REGISTRY_CA_CERT := REGISTRY_TLS_DIR + "/ca.crt"

export OPENSSL_CONFIG := '''
[req]
distinguished_name=req_distinguished_name
req_extensions=v3_req
prompt=no

[req_distinguished_name]
CN=kuberaid-registry

[v3_req]
subjectAltName=@alt_names

[alt_names]
IP.1=10.0.2.2
DNS.1=localhost
'''

# =========================
# Talos VM Module
# =========================

TALOS_VERSION := "v1.13.2"
VM_NAME := "talos-node"
SYSTEM_DISK := lab_dir + "/talos.qcow2"
DATA_DISK := lab_dir + "/data.qcow2"
ISO := lab_dir + "/talos-" + TALOS_VERSION + ".qcow2"
RAM := "4096"
CPUS := "2"

secrets_yaml := gen_dir + '/secrets.yaml'

CONTAINER := "podman"
REGISTRY_PORT := "5000"
REGISTRY_NAME := "kuberaid-registry"
REGISTRY_IMAGE := "registry:2"
REGISTRY_ADDR := "10.0.2.2:" + REGISTRY_PORT

NODE_IP := "127.0.0.1"
CONTROL_PLANE_ADDR := NODE_IP + ":50000"
KUBE_API_ADDR := NODE_IP + ":6443"

export KUBECONFIG := gen_dir + "/kubeconfig"
export TALOSCONFIG := gen_dir + "/talosconfig"

[private]
container cmd *args:
    {{CONTAINER}} {{cmd}} {{args}}

[private]
schematic-id:
    curl -X POST --data-binary @"{{ lab_dir }}/schematic.yaml" "https://factory.talos.dev/schematics" 2>/dev/null | jq -r '.id'

# -------------------------
# Download Talos ISO
# -------------------------
download-image:
    just log "info" "Downloading Talos ISO"
    if [[ ! -f "{{ISO}}" ]]; then \
        curl -L -o "{{ISO}}" \
            "https://factory.talos.dev/image/$(just lab schematic-id)/{{TALOS_VERSION}}/metal-amd64.qcow2"; \
    else \
        just log "info" "ISO already exists, skipping"; \
    fi

# -------------------------
# Create VM disk
# -------------------------
create-disk disk:
    just log "info" "Creating VM disk"
    if [[ ! -f "{{ disk }}" ]]; then \
        qemu-img create -f qcow2 "{{ disk }}" 20G; \
    else \
        just log "info" "{{ disk }} already exists, skipping"; \
    fi

# -------------------------
# Create raw disk
# -------------------------
create-raw disk:
    just log "info" "Creating VM disk"
    if [[ ! -f "{{ disk }}" ]]; then \
        dd if=/dev/zero of="{{ disk }}" bs=1024M count=20; \
    else \
        just log "info" "{{ disk }} already exists, skipping"; \
    fi

# -------------------------
# Map disk image
# -------------------------
losetup disk: (create-raw disk)
    just log "info" "Setup loop"

    if [[ -n "$(losetup -j "{{ disk }}" -O NAME --noheadings 2>/dev/null || true)" ]]; then \
        just log "info" "Already attached: {{ disk }}"; \
    else \
        just log "info" "Creating loop device"; \
        sudo losetup -f "{{ disk }}" --direct-io=on --show; \
    fi

import: (losetup "disk.img")
    just log "info" "Import pool"
    sudo zpool import lab

# -------------------------
# Start VM
# -------------------------
up: download-image (create-disk DATA_DISK)
    just log "info" "Starting Talos VM"

    if [[ ! -f "{{ SYSTEM_DISK }}" ]]; then \
        cp -ar "{{ISO}}" "{{SYSTEM_DISK}}"; \
    else \
        just log "info" "{{ SYSTEM_DISK }} already exists, skipping"; \
    fi

    qemu-system-x86_64 \
        -enable-kvm \
        -m {{RAM}} \
        -smp {{CPUS}} \
        -cpu host \
        -name "{{VM_NAME}}" \
        -drive file="{{SYSTEM_DISK}}",format=qcow2 \
        -drive file="{{DATA_DISK}}",format=qcow2 \
        -cdrom "{{ISO}}" \
        -netdev user,id=net0,hostfwd=tcp::50000-:50000,hostfwd=tcp::6443-:6443 \
        -device virtio-net-pci,netdev=net0 \
        -display none \
        -serial file:serial.log \
        -daemonize

# -------------------------
# Wait for API
# -------------------------
talos-wait:
    just log "info" "Waiting for Talos API..."
    # until talosctl version --insecure --nodes "{{ NODE_IP }}" >/dev/null 2>&1; do \
    #     sleep 2; \
    # done

# -------------------------
# Generate minimal config
# -------------------------
talos-gen-config:
    just log "info" "Generating Talos controlplane config"
    mkdir -p "{{ gen_dir }}"
    if [[ ! -f "{{ secrets_yaml }}" ]]; then \
        talosctl gen secrets -o "{{ secrets_yaml}}"; \
    else \
        just log "info" "secrets.yaml already exists, skipping"; \
    fi
    
    talosctl gen config kuberaid-lab "https://{{KUBE_API_ADDR}}" \
        --with-secrets "{{ secrets_yaml }}" \
        --config-patch-control-plane @"{{ lab_dir }}/controlplane.yaml" \
        --additional-sans "{{ NODE_IP }}" \
        --output-types "controlplane,talosconfig" \
        --output "{{ gen_dir }}" \
        --force
    talosctl config endpoint "https://{{CONTROL_PLANE_ADDR}}"
    talosctl config node "{{NODE_IP}}"

# -------------------------
# Apply config
# -------------------------
talos-apply-config *args: talos-gen-config talos-wait
    just log "info" "Applying Talos config"

    talosctl apply-config \
        --nodes "{{ NODE_IP }}" \
        --file "{{ gen_dir }}/controlplane.yaml" {{ args }}


# -------------------------
# Bootstrap cluster
# -------------------------
talos-bootstrap:
    just log "info" "Bootstrapping cluster (etcd init)"

    talosctl bootstrap \
        --nodes "{{ NODE_IP }}"


# -------------------------
# Fetch kubeconfig
# -------------------------
talos-kubeconfig:
    just log "info" "Fetching kubeconfig"

    talosctl kubeconfig \
        --nodes "{{ NODE_IP }}" \
        {{KUBECONFIG}}


# -------------------------
# Full bring-up
# -------------------------
up-full: up (talos-apply-config "--insecure") talos-bootstrap talos-kubeconfig
    just log "info" "Talos cluster ready"

# -------------------------
# Stop VM (best effort)
# -------------------------
down:
    just log "warn" "Stopping Talos VM (best effort)"
    pkill -f "{{VM_NAME}}" || true


# -------------------------
# Reset everything
# -------------------------
reset: down
    just log "warn" "Deleting VM state"
    rm -f "{{SYSTEM_DISK}}" "{{DATA_DISK}}" controlplane.yaml {{KUBECONFIG}}

registry-gen-certs:
    just log "info" "Generating self-signed registry TLS certs"
    mkdir -p "{{REGISTRY_TLS_DIR}}"

    # CA
    openssl genrsa -out "{{REGISTRY_CA_KEY}}" 4096

    openssl req -x509 -new -nodes \
        -key "{{REGISTRY_CA_KEY}}" \
        -sha256 -days 3650 \
        -subj "/CN=kuberaid-registry-ca" \
        -out "{{REGISTRY_CA_CERT}}"

    # Server cert config
    printf %s "$OPENSSL_CONFIG" > "{{REGISTRY_TLS_DIR}}/openssl.cnf"

    # Server key
    openssl genrsa -out "{{REGISTRY_KEY}}" 4096

    # CSR
    openssl req -new \
        -key "{{REGISTRY_KEY}}" \
        -out "{{REGISTRY_TLS_DIR}}/domain.csr" \
        -config "{{REGISTRY_TLS_DIR}}/openssl.cnf"

    # Sign cert
    openssl x509 -req \
        -in "{{REGISTRY_TLS_DIR}}/domain.csr" \
        -CA "{{REGISTRY_CA_CERT}}" \
        -CAkey "{{REGISTRY_CA_KEY}}" \
        -CAcreateserial \
        -out "{{REGISTRY_CERT}}" \
        -days 3650 \
        -sha256 \
        -extensions v3_req \
        -extfile "{{REGISTRY_TLS_DIR}}/openssl.cnf"

registry-up: registry-gen-certs
    just log "info" "Starting TLS container registry"

    {{CONTAINER}} ps -a --format '{{"{{"}}.Names{{"}}"}}' | grep -q "{{REGISTRY_NAME}}" && \
        {{CONTAINER}} start "{{REGISTRY_NAME}}" || \
        {{CONTAINER}} run -d \
            --restart=always \
            -p {{REGISTRY_PORT}}:5000 \
            --name "{{REGISTRY_NAME}}" \
            -v "{{REGISTRY_TLS_DIR}}:/certs" \
            -e REGISTRY_HTTP_TLS_CERTIFICATE=/certs/domain.crt \
            -e REGISTRY_HTTP_TLS_KEY=/certs/domain.key \
            "{{REGISTRY_IMAGE}}"


registry-down:
    just log "warn" "Stopping registry"
    {{CONTAINER}} stop "{{REGISTRY_NAME}}" || true

registry-reset: registry-down
    just log "warn" "Removing registry container"
    {{CONTAINER}} rm "{{REGISTRY_NAME}}" || true

deploy:
    just log "info" "Deploy"
    cargo clippy
    just container push manager
    just helm apply \
        -n kube-system \
        --set-literal=controllers.manager.containers.main.image.repository="{{REGISTRY_ADDR}}/kuberaid-manager" \
        --set-literal=controllers.manager.containers.main.image.tag="dev" \
        --set-literal=controllers.manager.containers.main.image.pullPolicy="Always" \
        --set-literal=controllers.csi-driver.containers.main.image.repository="{{REGISTRY_ADDR}}/kuberaid-manager" \
        --set-literal=controllers.csi-driver.containers.main.image.tag="dev" \
        --set-literal=controllers.csi-driver.containers.main.image.pullPolicy="Always"

watch:
    cargo watch -s "just lab deploy" -i "**/crd.yaml" -d 5
