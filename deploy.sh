#!/bin/sh

cargo build
cd frontend
npm run build
cd ..
ssh yhs@big-sign.local "sudo systemctl stop big-sign"
scp target/arm64-unknown-linux-gnu/debug/yhs-sign yhs@big-sign.local:~
scp -r static yhs@big-sign.local:~
ssh yhs@big-sign.local "sudo systemctl start big-sign"