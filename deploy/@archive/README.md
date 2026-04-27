Previous deployment attempts.

Version 1 used podman and built on the device. Somewhat annoying to update.

Version 2 used full nix. Also annoying to update, signing packages and doing nixos rebuilds with each iteration.

Version 3 just pushed the binary. This worked with raspberrypi4 probably because there were artifacts from prior rust builds on the device, but didn't work with raspberrypi5.

Next version:

Attempting a combo of 3 and 2

nix-build to cross compile locally
`nix copy --to ssh://user@remote-machine nixpkgs#package-name` or something like it to copy to the remote store
create/update a symlink in the previous place where the binary was