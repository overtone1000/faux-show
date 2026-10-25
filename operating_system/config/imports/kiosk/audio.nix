{ pkgs, ... }:

{
  # Need to have alsa-lib package installed at system level, not just as a build dependency?
  environment.systemPackages = with pkgs; [ 
    alsa-lib
  ];
}