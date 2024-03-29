#!/usr/bin/env bash

# thanks https://stackoverflow.com/questions/4774054/reliable-way-for-a-bash-script-to-get-the-full-path-to-itself
SCRIPT=$(realpath "$0")
SCRIPTPATH=$(dirname "$SCRIPT")

PROD_CONF_FILE=".env"

if [ ! -f "$PROD_CONF_FILE" ]; then
    echo "could not find $PROD_CONF_FILE"
    exit 1
fi

source .env

source $SCRIPTPATH/verify_requirements.sh

sqlx mig run