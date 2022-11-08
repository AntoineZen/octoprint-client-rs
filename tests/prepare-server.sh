#!/bin/bash

DIR=$(dirname "$0")

cd $DIR

unzip data.zip
mkdir -p data/octoprint
mv -f basedir/* data/octoprint

rm -r basedir
rm metadata.json

docker-compose up -d