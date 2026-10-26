# Pi 5

## Procedure

https://github.com/nvmd/nixos-raspberrypi

**As with all SOC installations, make sure power supply to device and any peripherals like a monitor is adequate!**

To try to compile on desktop, need to emulate native compilation. Add this to config:
`boot.binfmt.emulatedSystems = [ "aarch64-linux" ];`

Although emulated native compilation is slow, there will hopefully be enough cached that it doesn't take too long.

**Previously did this with emulation on system in misc.nix (boot.binfmt.emulatedSystems = [ "aarch64-linux" ];) but this is pretty slow for builds. Can do with cross-compile instead? See rpi5_kiosk. Can even mix them by only adding cross settings to the specific import.**
**Using the cross compiler compiles _everthing_ because cached packages aren't available. Pretty slow.**

### Installer build
```
nixpkgs.hostPlatform = "aarch64-linux";
nixpkgs.buildPlatform = "x86_64-linux";
```

Build an installer image with:
`nix --extra-experimental-features nix-command --extra-experimental-features flakes build github:nvmd/nixos-raspberrypi#installerImages.rpi5 --system aarch64-linux`

Write it to a USB (not SD)
```
DEVICE=/dev/sdc #whichever device is the target, find with `lsblk`
IMAGE=./result/sd-image/nixos-installer-rpi5-kernel.img.zst
sudo umount $DEVICE*
sudo nix-shell -p zstd --run "zstdcat $IMAGE | dd of=$DEVICE bs=4M status=progress conv=fsync"
```

### Installation

Use nixos-anywhere like described in the nixos-raspberrypi README.

Notes about installation:
- Portable monitors don't work as well. RPi should show display during boot. Connect to a full monitor to see output and troubleshoot.
- disko works well. It's worth it to configure the filesystems during initial installation.
- nixos-anywhere has some requirements such as sudo without password for the installation account. All the instructions show or being done with root! Best to do so.
- A proven working installation flake is in this directory.

Procedure:
- Boot to USB running installer image
- Insert SD card
- Determine SD card device name with lsblk (should be the one without the root or nix store mountpoints, /dev/mmcblk0 in first case)
- Set flake.nix to use the installation user config required by nixos-anywhere
- Change flake.nix file system argument to the SD card device name
```
nix-shell -p nixos-anywhere
nixos-anywhere --flake /etc/nixos/trm_nixos/devices/rpi5_kiosk#rpi5 root@$DEVICE_IP
```

## First Config Update

- Will need to update using a modified version of the flake. Initial update will need to be with root, but user configuration changes will then apply.

`sudo nixos-rebuild switch --no-reexec --flake /path/to/flake#hostname --target-host root@target-ip`

So, for example
`sudo nixos-rebuild switch --no-reexec --flake /etc/nixos/trm_nixos/devices/rpi5_kiosk/#rpi5 --target-host root@$DEVICE_IP`

Once initial update is done, should be able to do it with normal user
`sudo nixos-rebuild switch --no-reexec --flake /etc/nixos/trm_nixos/devices/rpi5_kiosk/#rpi5 --sudo --target-host $DEV_USER@$DEVICE_IP`