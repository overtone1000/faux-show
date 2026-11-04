This directory contains training data for rustpotter.

## Build
Build for nix with `nix-build rustpotter-cli.nix`

## Run
Run from repo root with `./experimental/voice_control/rustpotter/result/bin/rustpotter-cli`

## Recording Samples
`./experimental/voice_control/rustpotter/result/bin/rustpotter-cli record ./experimental/voice_control/rustpotter/samples/[faux-show]0.wav`
`./experimental/voice_control/rustpotter/result/bin/rustpotter-cli record ./experimental/voice_control/rustpotter/samples/negative-example.wav`

Move them to training or testing, but it's best to have separate sets for each

Also, it's best if the training files are about the same length or they'll be truncated or padded and make training take longer.

## Building Model
```
./experimental/voice_control/rustpotter/result/bin/rustpotter-cli train \
-t small \
--train-dir ./experimental/voice_control/rustpotter/samples/training \
--test-dir ./experimental/voice_control/rustpotter/samples/testing \
--test-epochs 10 \
--epochs 2500 \
-l 0.017 \
./experimental/voice_control/rustpotter/models/small.rpw
```