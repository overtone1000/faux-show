## Updating

In the repo root, run:
`sudo nixos-rebuild switch --ask-sudo-password --no-reexec --flake ./operating_system/v1_nvmd/config#rpi5 --sudo --target-host $USER@$DEVICE_IP`

May need
`sudo nix --extra-experimental-features 'nix-command flakes' flake update`
to force some things to update.
    -Needed to update kiosk user UID
    -Needed to update systemd service? Or is __disabling import of service and then reenabling__ is what actually did it? Did both simultaneously, so not sure.

May need to redeploy the software? Does rebuilding affect this package?

## Configuration

### Wifi (wpa supplicant wireless configuration)

Wifi network configuration is best achieved with `iwctl` tool.
```
iwctl
device list
station $DEVICE_NAME connect $SSID
station $DEVICE_NAME show
station $DEVICE_NAME get-networks
station $DEVICE_NAME connect-hidden $SSID
known-networks list
known-networks $SSID forget
```

## Build Failing

libxkbcommon-1.13.1 now fails to build. This is probably a cross-compilation problem. Tried disabling much of the flake and including new packages.