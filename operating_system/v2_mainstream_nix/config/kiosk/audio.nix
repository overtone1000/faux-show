{ pkgs, ... }:

{
  # Need to have alsa-lib package installed at system level, not just as a build dependency?
  environment.systemPackages = with pkgs; [ 
    alsa-lib
    alsa-utils
  ];

  # Enable sound.
  #services.pulseaudio.enable = true;
  #OR
  security.rtkit.enable = true;
  services.pipewire = {
    enable = true;
    alsa.enable = true;
    alsa.support32Bit = true;
    pulse.enable = true;
    wireplumber.enable = true;
    systemWide = true;
  };

  systemd.user.services.wireplumber.wantedBy = [ "default.target" ];

  users.users.kiosk = {
    #linger = true; #Tried this but this appears to break kiosk mode. User should be logged in when running, so linger shouldn't be necessary.
    extraGroups = [ "audio" ];
  };

  #hardware.raspberry-pi.configtxt.settings = {
  #  all = {
  #    dtparam = [ "audio=on" ];
  #  };
  #};
}