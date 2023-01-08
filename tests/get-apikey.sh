#!/bin/bash

DIR=$(dirname "$0")

DATA_ARCHIVE="$DIR/data.zip"

#unzip -p $DIR/data.zip basedir/config.yaml | yq .api.key
unzip -p $DATA_ARCHIVE basedir/users.yaml | yq .rust.apikey