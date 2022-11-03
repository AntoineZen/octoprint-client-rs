#!/bin/bash

unzip data.zip
mkdir -p data/octoprint
mv basedir/* data/octoprint

rm -r basedir
rm metadata.json

docker-compose up