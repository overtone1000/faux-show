## Updating

In the repo root, run:
`sudo nixos-rebuild switch --ask-sudo-password --no-reexec --flake ./operating_system/config#rpi5 --sudo --target-host $USER@$DEVICE_IP`

May need
`sudo nix --extra-experimental-features 'nix-command flakes' flake update`
to force some things to update.
    -Needed to update kiosk user UID
    -Needed to update systemd service? Or is __disabling import of service and then reenabling__ is what actually did it? Did both simultaneously, so not sure.

May need to redeploy the software? Does rebuilding affect this package?

Having new build problems even though nix config hasn't changed. Ran the update command above from the flake directory.
Definitely caused a full rebuild.

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