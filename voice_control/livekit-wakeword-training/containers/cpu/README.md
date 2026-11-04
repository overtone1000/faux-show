## Build the training image in this directory. **This takes a long time to build and will stop for several long periods without command line output!**
```
podman build -t livekit-wakeword-trainer .
```

## Storage
- A volume contains livekit data
- /home/tyler/livekit contains mountpoints
    - ./config is just a symbolic link to the configs dir in this repo
    - ./livekit_output will contain the results of an individual run, so it'd be good to copy pertinent parts back tot he results here!

## Run container

```
podman run -it \
    --mount=type=volume,source=livekit_data,dst=/data \
    --mount=type=bind,source=/home/tyler/livekit/configs,dst=/configs \
    --mount=type=bind,source=/home/tyler/livekit/livekit_output,dst=/output \
    livekit-wakeword-trainer bin/bash
```

## Creating a new model
- Create a new config in `configs`
- _Be sure_ to update the model name so it is built in a separate directory in output!

## Config and Training
Choose config
    ```
    SELECTED_CONFIG=name_of_config.yaml
    ```

Set up with the following command. This will download a huge payload. Dozens of gigabytes!!
    ```
    livekit-wakeword setup --config configs/$SELECTED_CONFIG
    ```

Train with:
    ```
    livekit-wakeword run configs/$SELECTED_CONFIG
    ```

Depending on what changed, it may be much faster to run training step-by-step
    ```
        livekit-wakeword generate configs/$SELECTED_CONFIG  # TTS synthesis + adversarial negatives
        livekit-wakeword augment configs/$SELECTED_CONFIG   # Augment + extract features
        livekit-wakeword train configs/$SELECTED_CONFIG     # 3-phase adaptive training
        livekit-wakeword export configs/$SELECTED_CONFIG    # Export to ONNX (default)
        livekit-wakeword eval configs/$SELECTED_CONFIG     # Evaluate model (DET curve, AUT, FPPH)
    ```

Don't forget to check the eval output (especially the png) to get optimal threshold!