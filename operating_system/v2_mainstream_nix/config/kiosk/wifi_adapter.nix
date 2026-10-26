
{ config, ... }:

{
  # Realtek rtl88x2bu driver for the TPLink Archer T3U
  # Not needed for kernel >= 6.12, which is what this device is on, so this shouldn't be necessary.
  # boot.extraModulePackages = [ config.boot.kernelPackages.rtl88x2bu ];
}