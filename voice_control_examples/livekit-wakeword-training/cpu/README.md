## Build the training image in this directory. **This takes a long time to build and will stop for several long periods without command line output!**
```
podman build -t livekit-wakeword-trainer .
```


NEVER UPDATED CONFIGS DIRECTORY, PROD.YAML hasn't been getting updated!!!

## Run container

Test
```
podman run -it \
    --mount=type=volume,source=livekit_data,dst=/data \
    --mount=type=bind,source=/home/tyler/livekit/livekit_output_test,dst=/output \
    livekit-wakeword-trainer bin/bash
```

Production
```
podman run -it \
    --mount=type=volume,source=livekit_data,dst=/data \
    --mount=type=bind,source=/home/tyler/livekit/livekit_output,dst=/output \
    livekit-wakeword-trainer bin/bash
```

## Config and Training
Set up with the following command. This will download a huge payload. Dozens of gigabytes!!

```
livekit-wakeword setup --config configs/prod.yaml
```

Train with:
```
livekit-wakeword run configs/prod.yaml
```

Depending on what changed, it may be much faster to run training step-by-step
```
    livekit-wakeword generate configs/prod.yaml  # TTS synthesis + adversarial negatives
    livekit-wakeword augment configs/prod.yaml   # Augment + extract features
    livekit-wakeword train configs/prod.yaml     # 3-phase adaptive training
    livekit-wakeword export configs/prod.yaml    # Export to ONNX (default)
    livekit-wakeword eval configs/prod.yaml      # Evaluate model (DET curve, AUT, FPPH)
```

Don't forget to check the eval output (especially the png) to get optimal threshold!