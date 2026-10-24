
{ config, pkgs, ... }:

{
  # Realtek rtl88x2bu driver for the TPLink Archer T3U
  boot.extraModulePackages = [ config.boot.kernelPackages.rtl88x2bu ];
}