# Edit this configuration file to define what should be installed on
# your system.  Help is available in the configuration.nix(5) man page
# and in the NixOS manual (accessible by running ‘nixos-help’).

{ config, lib, nixpkgs, ... }:

let
	crosspkgs=import nixpkgs {system="aarch64-linux";};
in
{
	options.mymodule.import_pkgs = lib.mkOption {
		type = lib.types.anything;
	};


	imports =
	[ 
		./kiosk/users.nix
		./kiosk/faux-show-backend.nix
				
		##Import explicitly to avoid leaking options/arguments. Without the import, this builds firefox.
		##../../../commons/kiosk/kiosk.nix
		(
			import ./kiosk/kiosk.nix
			{
				pkgs=crosspkgs;
				config=config;
			}
		)

		#Build failing, comment out wifi adapter to determine cause.
		# Not needed for kernel >= 6.12 anyway, which is what this device is on, so this shouldn't be necessary. Just try configuring the adapter without this import.
		#(
		#	import ./kiosk/wifi_adapter.nix
		#	{
		#		config=config;
		#	}
		#)

		#Build failing, comment out audio for now to try to determine the cause. Still did not build with audio commented out.
		#(
		#	import ./kiosk/audio.nix
		#	{
		#		pkgs=crosspkgs;
		#	}
		#)

		##Instead of building rust app with nix, just install toolchain (rustc and cargo) and see if that allows binaries to run
		##Nope, it don't work
		#(
		#	import ../../../commons/kiosk/nix-rust.nix
		#	{
		#		pkgs=crosspkgs;
		#		config=config;
		#	}
		#)
	];
}
