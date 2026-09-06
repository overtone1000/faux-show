## Build the training image in this directory. **This takes a long time to build and will stop for several long periods without command line output!**
```
podman build -t livekit-wakeword-trainer .
```

## GPU Config
Observe GPU utilization with
`nix-shell -p nvitop --run nvitop`
Or run `nvidia-smi`

Trying stuff at https://discourse.nixos.org/t/nvidia-gpu-support-in-podman-and-cdi-nvidia-ctk/36286/9

Generate nvidia config
`nvidia-ctk cdi generate --output nvidia.yaml`

Can confirm GPU is visible in container with `nvidia-smi -L`

Can confirm CUDA is available with `python3 -c "import torch; print('CUDA Available:', torch.cuda.is_available())"`

## Run container

--device=/dev/dri/renderD128:/dev/dri/renderD128:rw \
--device=/dev/dri/card1:/dev/dri/card1:rw \

```
podman run -it \
    --mount=type=volume,source=livekit_data,dst=/data \
    --mount=type=bind,source=/home/tyler/livekit_output,dst=/output \
    --device nvidia.com/gpu=all \
    livekit-wakeword-trainer /bin/bash
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