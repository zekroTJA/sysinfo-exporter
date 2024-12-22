#!/bin/bash

set -ex

tag=$(git describe --tag --abbrev=0)

sed 's/<<VERSION>>/'"${tag:1}"'/' -i DEBIAN/control

dpkg-deb --build .