# Nix Deployment

## Overview

This build approach does the following:
1. Cross compile and package the backend for Nix using Crane via `flake.nix`.
2. Build the frontend.
3. Copy the resulting package to the device with `nix copy`.
4. Set the environment file on the device.
5. Copy the frontend to the device's `/var/lib/www/` directory for use by the backend.

Running the backend as a systemd service is in the _config_ of the device.


## Initializing for Deployment

The build machine must be configured as a nix cache. First, generate a key pair for signing packages:
```
nix-store --generate-binary-cache-key builder ./deploy/nix/nix-store-binary-cache-key-{secret,public}
```

On the remote host, the configurations must be modified to include the public key as a trusted cache.
```
nix.settings.trusted-public-keys = [
  <content of nix-store-binary-cache-key-public>
];
```

## Deploying

Deploy from the repo root with:

```
bash ./deploy/scripts/01_build_backend.sh
bash ./deploy/scripts/02_build_frontend.sh
bash ./deploy/scripts/03_push_to_device.sh
```

### Restart after backend update
```
sudo systemctl restart faux-show-backend cage-tty1
```