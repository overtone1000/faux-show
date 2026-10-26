# Edit this configuration file to define what should be installed on
# your system.  Help is available in the configuration.nix(5) man page
# and in the NixOS manual (accessible by running ‘nixos-help’).

{ config, lib, nixpkgs, ... }:

{

	imports =
	[ 
		./ssh.nix
		./kiosk/users.nix
		./kiosk/faux-show-backend.nix
		./kiosk/kiosk.nix
		./kiosk/audio.nix
	];
}
