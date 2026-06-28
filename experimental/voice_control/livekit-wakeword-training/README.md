Build the training image in this directory. **This takes a long time to build and will stop for several long periods without command line output!**
```
podman build -t livekit-wakeword-trainer .
```

```
podman run -it \
    --mount=type=volume,source=livekit_data,dst=/data \
    --mount=type=bind,source=/home/tyler/livekit_output,dst=/output \
    livekit-wakeword-trainer /bin/bash
```

Set up with the following command. This will download a huge payload. Dozens of gigabytes!!

```
livekit-wakeword setup --config configs/prod.yaml
```

Train with:
```
livekit-wakeword run configs/prod.yaml
```