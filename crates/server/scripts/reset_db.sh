#!/usr/bin/env bash

# thanks https://stackoverflow.com/questions/4774054/reliable-way-for-a-bash-script-to-get-the-full-path-to-itself
SCRIPT=$(realpath "$0")
SCRIPTPATH=$(dirname "$SCRIPT")

source $SCRIPTPATH/verify_requirements.sh

sqlx database drop
sqlx database create
sqlx mig run