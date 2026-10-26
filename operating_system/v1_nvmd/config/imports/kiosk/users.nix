
{ config, pkgs, ... }:
let
    universal_keys = [ 
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILhHNqrrHT1SVQLrw0h3jxbB+eUG1Bskxpho1PAP7P1j tyler@nixos"
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILtzYVbugD5KBJVcKlI2ShK4umWf4UzxhywXEF/qImN6 tyler@nixos"
    ];
in
{

  users.users.root = {
    openssh.authorizedKeys.keys  = universal_keys;
  };
  
  users.users.tyler = {
    openssh.authorizedKeys.keys  = universal_keys;
  };

  nix.settings.trusted-users = [ "tyler" ];
	users.users = {
		tyler = {
			isNormalUser = true;
			description = "Tyler";
			extraGroups = [ 
				"networkmanager"
				"wheel" #root privileges
			];
		};
	};
}
