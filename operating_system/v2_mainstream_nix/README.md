After v1 was developed, NixoS released support for the raspberry pi 5 on `nixos-unstable`.
https://wiki.nixos.org/wiki/NixOS_on_ARM/Raspberry_Pi_5

At the same time, the v1 flake stopped building successfully due to failing tests, and the reason for those errors remain unclear, and no solution has yet been found.

## Initial Installation
https://wiki.nixos.org/wiki/NixOS_on_ARM/Installation#SD_card_images_(SBCs_and_similar_platforms)
*hydra.nixos.org apparently only works with chrome now*

Flash correct image to SD card and boot

`lsblk` to find the SD card
```
IMAGE=./operating_system/v2_mainstream_nix/images/nixos-image-sd-card-26.11pre1102304.798aaac784d0-aarch64-linux.img.zst
SD_CARD=/dev/SDWHATEVER
nix-shell -p zstd --run "zstdcat $IMAGE | sudo dd of=$SD_CARD status=progress bs=4M status=progress conv=fsync"
```

Modify the image to add ssh key to root account to start with. This will make configuration changes much easier.
```
PUBSSH=~/.ssh/id_ed25519.pub && \
ROOTDIR=/run/media/tyler/NIXOS_SD1/root && \
sudo mkdir -p $ROOTDIR/.ssh && \
sudo chmod 700 $ROOTDIR/.ssh && \
sudo cat $PUBSSH | sudo tee $ROOTDIR/.ssh/authorized_keys && \
sudo chmod 600 $ROOTDIR/.ssh/authorized_keys
```

SSH in and generate initial config with `nixos-generate-config`
Modify the configuration.nix to import from "config/imports.nix"

## Configuration updates
Just make changes in `config` directory and run `update_config.sh`

## Wifi (wpa supplicant wireless configuration)

`nmtui` to configure wifi