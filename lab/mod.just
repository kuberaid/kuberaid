set positional-arguments := true
set quiet := true
set shell := ['bash', '-euo', 'pipefail', '-c']

lab_dir := justfile_dir() + '/lab'

# =========================
# Talos VM Module
# =========================

TALOS_VERSION := "v1.12.6"
VM_NAME := "talos-node"
SYSTEM_DISK := lab_dir + "/talos.qcow2"
DATA_DISK := lab_dir + "/data.qcow2"
ISO := lab_dir + "/talos-" + TALOS_VERSION + ".iso"
RAM := "4096"
CPUS := "2"
CONTROL_PLANE_ADDR := "127.0.0.1:50000"
KUBE_API_ADDR := "127.0.0.1:6443"

export KUBECONFIG := lab_dir + "/kubeconfig"
export TALOSCONFIG := lab_dir + "/talosconfig"


# -------------------------
# Download Talos ISO
# -------------------------
talos-download-iso:
    just log "info" "Downloading Talos ISO"
    if [[ ! -f "{{ISO}}" ]]; then \
        curl -L -o "{{ISO}}" \
            "https://github.com/siderolabs/talos/releases/download/{{TALOS_VERSION}}/metal-amd64.iso"; \
    else \
        just log "info" "ISO already exists, skipping"; \
    fi

# -------------------------
# Create VM disk
# -------------------------
talos-create-disk disk:
    just log "info" "Creating VM disk"
    if [[ ! -f "{{ disk }}" ]]; then \
        qemu-img create -f qcow2 "{{ disk }}" 20G; \
    else \
        just log "info" "{{ disk }} already exists, skipping"; \
    fi


# -------------------------
# Start VM
# -------------------------
talos-up: talos-download-iso
    just log "info" "Starting Talos VM"
    just lab talos-create-disk "{{ SYSTEM_DISK }}"
    just lab talos-create-disk "{{ DATA_DISK }}"

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
        -daemonize


# -------------------------
# Wait for API
# -------------------------
talos-wait:
    just log "info" "Waiting for Talos API..."
    # until talosctl version --insecure --nodes "{{CONTROL_PLANE_ADDR}}" >/dev/null 2>&1; do \
    #     sleep 2; \
    # done
    until nc -z 127.0.0.1 50000; do \
        sleep 1; \
    done

# -------------------------
# Generate minimal config
# -------------------------
talos-gen-config:
    just log "info" "Generating Talos controlplane config"
    talosctl gen config kuberaid-lab "https://{{KUBE_API_ADDR}}" \
        --additional-sans "127.0.0.1" \
        --output-types "controlplane,talosconfig" --force
    talosctl config endpoint "https://{{CONTROL_PLANE_ADDR}}"
    talosctl config node "{{CONTROL_PLANE_ADDR}}"

# -------------------------
# Apply config
# -------------------------
talos-apply-config: talos-gen-config talos-wait
    just log "info" "Applying Talos config"

    talosctl apply-config \
        --insecure \
        --nodes {{CONTROL_PLANE_ADDR}} \
        --file controlplane.yaml


# -------------------------
# Bootstrap cluster
# -------------------------
talos-bootstrap:
    just log "info" "Bootstrapping cluster (etcd init)"

    talosctl bootstrap \
        --nodes "127.0.0.1"


# -------------------------
# Fetch kubeconfig
# -------------------------
talos-kubeconfig:
    just log "info" "Fetching kubeconfig"

    talosctl kubeconfig \
        --nodes "127.0.0.1" \
        {{KUBECONFIG}}


# -------------------------
# Full bring-up
# -------------------------
talos-up-full: talos-up talos-apply-config talos-bootstrap talos-kubeconfig
    just log "info" "Talos cluster ready"

# -------------------------
# Stop VM (best effort)
# -------------------------
talos-down:
    just log "warn" "Stopping Talos VM (best effort)"
    pkill -f "{{VM_NAME}}" || true


# -------------------------
# Reset everything
# -------------------------
talos-reset: talos-down
    just log "warn" "Deleting VM state"
    rm -f "{{SYSTEM_DISK}}" "{{DATA_DISK}}" controlplane.yaml {{KUBECONFIG}}
