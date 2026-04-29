## Nix Deployment

This build approach does the following:
1. Cross compile and package the backend for Nix using Crane via `flake.nix`.
2. Copy the resulting package to the device with `nix copy`.
3. Build the frontend.
4. Copy the frontend to the device's `/var/lib/www/` directory for use by the backend.

Running the backend as a systemd service is in the _config_ of the device. This is also where pertinent environment variables are set.