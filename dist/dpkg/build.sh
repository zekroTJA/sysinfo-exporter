#!/bin/bash

set -ex

tag=$(git describe --tag --abbrev=0)

sed 's/<<VERSION>>/'"${tag:1}"'/' -i debian/control

dpkg-deb --build .