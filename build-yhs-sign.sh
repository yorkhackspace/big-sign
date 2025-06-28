#!/usr/bin/env sh

set -e pipefail

outputDirectory="yhs-sign-deb/usr/bin"
outputPath="${outputDirectory}/yhs-sign"
rm -f $outputPath
mkdir -p $outputDirectory

nix build .#cross-arm64-linux
cp result/bin/yhs-sign $outputPath
chown $USER:$GROUP $outputPath
chmod 755 $outputPath
dpkg-deb --build yhs-sign-deb yhs-sign.deb
