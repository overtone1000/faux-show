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
      ExecStart = "/kiosk/faux-show-backend/bin/faux-show-backend";
      EnvironmentFile = "/kiosk/faux-show-environment/.env";
      Restart = "always";
      RestartSec = "15s";
      Type = "exec";
      User = "kiosk"; #needs to be run as kiosk as this is the run directory targeted in the backend
      Group = "audio";
      AmbientCapabilities= [ "CAP_NET_BIND_SERVICE "]; #allow binding of low ports
      SupplementaryGroups = [ "pipewire" ];
    };
  };
}
