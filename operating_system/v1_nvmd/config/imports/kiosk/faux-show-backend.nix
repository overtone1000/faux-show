{ config, pkgs, ... }:
{
  nix.settings.trusted-public-keys = [
    builder:+wDdLYeOirPqxBM4B2dv06mcpSim4UY7IqmvwqCtPmA=
  ];

  systemd.services."faux-show-backend" = {
    wants = [ 
      "network-online.target"
    ];
    requires = [
      "network.target"
    ];
    wantedBy = [
      "default.target"
    ];
    serviceConfig = {
      ExecStart = "/root/faux-show-backend/bin/faux-show-backend";
      EnvironmentFile = "/root/faux-show-environment/.env";
      Restart = "always";
      RestartSec = "15s";
      Type = "exec";
      User = "root"; #needs to be run as root
    };
  };
}
