#!/usr/bin/env bash
set -o nounset
set -o pipefail

# Checker: validate and check pipe manifest
# Entrypoint: main

# workspace
workspace="e2e"
# data directory
data_directory=""

function usage() { 
cat <<EOF
Checker
Options:
	-d | --directory
	path to workspace directory (default: e2e)
	-h | --help
	print usage
Usage:
	$0 -d </PATH/TO/WORKSPACE>
EOF
exit 1
}

function parse_args() {
	while [[ $# -gt 0 ]]
	do
		i="$1"
		case ${i} in
			-d|--directory)
			if [ $# -lt 2 ]; then
				usage
			fi
			workspace="$2"
			shift
			shift
			;;
			-h|--help)
			usage
			shift
			shift
			;;
			*)
			usage
			;;
		esac
	done
	if [ ! -d "${workspace}" ]; then
		echo "Directory ${workspace} not found, exit ..." 1>&2;
		exit 1;
	fi
    data_directory="${workspace}/data"
}

function setup() {
    mkdir -p ${data_directory}/etcd
    mkdir -p ${data_directory}/apps
    mkdir -p ${data_directory}/builds/apps
    mkdir -p ${data_directory}/builds/logs
    mkdir -p ${data_directory}/builds/restores
    mkdir -p ${data_directory}/manifests
}

function cleanup() {
    rm -rf ${data_directory}/etcd/*
    rm -rf ${data_directory}/apps/*
    rm -rf ${data_directory}/builds/apps/*
    rm -rf ${data_directory}/builds/logs/*
    rm -rf ${data_directory}/builds/restores/*
    rm -rf ${data_directory}/manifests/*
}

# Entrypoint of data volume setup script
function main() {
    parse_args $@
    cleanup
    setup
}

main $@