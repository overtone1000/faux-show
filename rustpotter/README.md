This directory contains training data for rustpotter.

## Build
Build for nix with `nix-build rustpotter-cli.nix`

## Run
Run from repo root with `./rustpotter/result/bin/rustpotter-cli`

## Recording Samples
`./rustpotter/result/bin/rustpotter-cli record ./rustpotter/samples/[faux-show]0.wav`
`./rustpotter/result/bin/rustpotter-cli record ./rustpotter/samples/negative-example.wav`

Move them to training or testing, but it's best to have separate sets for each

Also, it's best if the training files are about the same length or they'll be truncated or padded and make training take longer.

## Building Model
```
./rustpotter/result/bin/rustpotter-cli train \
-t small \
--train-dir ./rustpotter/samples/training \
--test-dir ./rustpotter/samples/testing \
--test-epochs 10 \
--epochs 2500 \
-l 0.017 \
./rustpotter/models/small.rpw
```