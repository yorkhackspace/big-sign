#!/bin/sh

ssh yhs@big-sign.local "sudo systemctl stop big-sign"
scp target/aarch64-unknown-linux-gnu/debug/yhs-sign yhs@big-sign.local:~
scp -r static yhs@big-sign.local:~/static
ssh yhs@big-sign.local "sudo systemctl start big-sign"