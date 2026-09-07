## Build the training image in this directory. **This takes a long time to build and will stop for several long periods without command line output!**
```
podman build -t livekit-wakeword-trainer-v2 --file Contairfile-v2 .
```

## Run container

```
podman run -it \
    --mount=type=volume,source=livekit-v2_data,dst=/data \
    --mount=type=bind,source=/home/tyler/livekit-v2_output,dst=/output \
    --device nvidia.com/gpu=all \
    livekit-wakeword-trainer-v2 /bin/bash
```

## GPU Config
Observe GPU utilization with
`nix-shell -p nvitop --run nvitop`
Or run `nvidia-smi`

Trying stuff at https://discourse.nixos.org/t/nvidia-gpu-support-in-podman-and-cdi-nvidia-ctk/36286/9

Generate nvidia config
`nvidia-ctk cdi generate --output nvidia.yaml`

Can confirm GPU is visible in container with `nvidia-smi -L`

`
    python3 -c "import torch; print('CUDA Available:', torch.cuda.is_available())"
    python3 -c "import torch; print(torch.randn(1).cuda())"
    python3 -c "import onnxruntime as ort; print(ort.get_available_providers())"
`

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

Last run looks like it finished augmenting.
```
livekit-wakeword train configs/prod.yaml && livekit-wakeword export configs/prod.yaml && livekit-wakeword eval configs/prod.yaml
```