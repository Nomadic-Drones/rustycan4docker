#!/usr/bin/env bash
set -euxo

# https://docs.docker.com/engine/extend/#developing-a-plugin

SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
PLUGIN_NAME="${PLUGIN_NAME:-nomadicdrones/rustycan4docker}"
IMAGE_NAME=rustycan4docker:plugin_build_stage
PLUGIN_DIR="${SCRIPT_DIR}"
SOURCE_DIR="${SCRIPT_DIR}"/..

# try remove the plugin first if exists
"${SCRIPT_DIR}"/uninstall-plugin.sh | true
sudo rm -rf "${PLUGIN_DIR}"/rootfs 

docker build -f "${PLUGIN_DIR}"/Dockerfile "${SOURCE_DIR}" -t "${IMAGE_NAME}"

id=$(docker create "${IMAGE_NAME}" true)
sudo mkdir -p "${PLUGIN_DIR}"/rootfs
sudo docker export "${id}" | sudo tar -x -C "${PLUGIN_DIR}"/rootfs

sudo docker plugin create ${PLUGIN_NAME} "${PLUGIN_DIR}"

docker rm -vf "${id}"
docker rmi -f "${IMAGE_NAME}"
sudo rm -rf "${PLUGIN_DIR}"/rootfs 

docker plugin ls