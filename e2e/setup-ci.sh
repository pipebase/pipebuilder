#!/usr/bin/env bash
set -o nounset
set -o pipefail

# Entrypoint: main

# workspace
workspace="e2e"
etcd="etcd.yml"
resources="resources"
repository="repository.yml"
scheduler="scheduler.yml"
builder="builder.yml"
api="api.yml"
sleep_period=5

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
		echo "Directory '${workspace}' not found, exit ..." 1>&2;
		exit 1;
	fi
    etcd="${workspace}/${etcd}"
    if [ ! -f "${etcd}" ]; then
        echo "Docker compose file '${etcd}' not found, exit ..." 1>&2;
        exit 1;
    fi
    resources="${workspace}/${resources}"
    repository="${resources}/${repository}"
    if [ ! -f "${repository}" ]; then
        echo "Repository config file '${repository}' not found, exit ..." 1>&2;
        exit 1;
    fi
    scheduler="${resources}/${scheduler}"
    if [ ! -f "${scheduler}" ]; then
        echo "Scheduler config file '${scheduler}' not found, exit ..." 1>&2;
        exit 1;
    fi
    builder="${resources}/${builder}"
    if [ ! -f "${builder}" ]; then
        echo "Builder config file '${builder}' not found, exit ..." 1>&2;
        exit 1;
    fi
    api="${resources}/${api}"
    if [ ! -f "${api}" ]; then
        echo "Api config file '${api}' not found, exit ..." 1>&2;
        exit 1;
    fi
}

function run_etcd() {
    docker-compose -f ${etcd} up -d
}

function run_repository() {
    RUST_LOG=info PIPEBUILDER_CONFIG_FILE=${repository} repository &
}

function run_scheduler() {
    RUST_LOG=info PIPEBUILDER_CONFIG_FILE=${scheduler} scheduler &
}

function run_builder() {
    RUST_LOG=info PIPEBUILDER_CONFIG_FILE=${builder} builder &
}

function run_api() {
    RUST_LOG=info PIPEBUILDER_CONFIG_FILE=${api} api &
}

function run_ci() {
    run_etcd
    sleep ${sleep_period}
    run_repository
    sleep ${sleep_period}
    run_scheduler
    sleep ${sleep_period}
    run_builder
    sleep ${sleep_period}
    run_api
    sleep ${sleep_period}
}

# Entrypoint to setup CI
function main() {
    parse_args $@
    run_ci
}

main $@
