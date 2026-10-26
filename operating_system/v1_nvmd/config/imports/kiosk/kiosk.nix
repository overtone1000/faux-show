
{ config, pkgs, ... }:

let
#  transparent-cursor = pkgs.fetchFromGitHub {
#    owner = "johnodon";
#    repo = "Transparent_Cursor_Theme";
#    rev = "22cf8e6b6ccbd93a7f0ff36d98a5b454f18bed77";
#    sha256 = "sha256-wf5wnSiJsDqcHznbg6rRCZEq/pUneRkqFIJ+mNWb4Go=";
#  };

  #To list input devices
  #nix-shell -p libinput
  #sudo libinput list-devices
  #udevadm info --attribute-walk --name=path-to-device
  
  #These are the udev rules for the 15.6" Ultra HD Tochscreen Portable Monitor
  udev_extrarules_large_monitor = ''
    SUBSYSTEM=="input", ATTRS{name}=="vc4-hdmi-0", ENV{LIBINPUT_IGNORE_DEVICE}="1"
    SUBSYSTEM=="input", ATTRS{name}=="vc4-hdmi-1", ENV{LIBINPUT_IGNORE_DEVICE}="1"
    SUBSYSTEM=="input", ATTRS{name}=="wch.cn TouchScreen", ATTRS{phys}=="usb-0000:01:00.0-1.4/input2", ENV{LIBINPUT_IGNORE_DEVICE}="1"
  '';

  #These are the udev rules for the smaller monitor
  udev_extrarules_small_monitor = ''
    SUBSYSTEM=="input", ATTRS{name}=="vc4-hdmi-0", ENV{LIBINPUT_IGNORE_DEVICE}="1"
    SUBSYSTEM=="input", ATTRS{name}=="vc4-hdmi-1", ENV{LIBINPUT_IGNORE_DEVICE}="1"
  '';
  #SUBSYSTEM=="input", ATTRS{name}=="vc4-hdmi-0", ENV{LIBINPUT_IGNORE_DEVICE}="1"
  #SUBSYSTEM=="input", ATTRS{name}=="vc4-hdmi-1", ENV{LIBINPUT_IGNORE_DEVICE}="1"
  #SUBSYSTEM=="input", ATTRS{name}=="wch.cn USB2IIC_CTP_CONTROL", ATTRS{phys}=="usb-0000:01:00.0-1.1/input0", ENV{LIBINPUT_IGNORE_DEVICE}="1"
in
{

  services.automatic-timezoned.enable=true;
  
  users.users = {
		kiosk = {
			isNormalUser = true;
			home = "/home/kiosk";
			description = "Kiosk";
      password = "";
      uid = 2000;
		};
	};

  #No display manager or desktop manager needed. Cage will handle this.
  #services.desktopManager.gnome.enable = true;
  #services.displayManager.gdm.enable=true;
  
  #services.xserver = {
  #  enable = true;
  #  videoDrivers = [ "fbdev" ];
  #  displayManager={
  #    #lightdm.enable = true; #couldn't get cage to auto login?
  #    gdm.enable = true;
  #  
  #    #Not working with lightdm, how about gdm?
  #    #autoLogin = {
  #    #  enable=true;
  #    #  user="kiosk";
  #    #};
  #  };
  #};

  #Necessary to autologin? Will cage do this on its own?
  #services.displayManager.autoLogin={
  #  enable=true;
  #  user="kiosk";
  #};

  # Some kind of workaround, https://nixos.wiki/wiki/GNOME
  #systemd.services={
  #  "getty@tty1".enable = false;
  #  "autovt@tty1".enable = false;
  #};


  # Enable sound with pipewire.
  services.pulseaudio.enable = false;
  security.rtkit.enable = true;
  services.pipewire = {
    enable = true;
    alsa.enable = true;
    alsa.support32Bit = true;
    pulse.enable = true;
    # If you want to use JACK applications, uncomment this
    #jack.enable = true;

    # use the example session manager (no others are packaged yet so this is enabled by default,
    # no need to redefine it in your config for now)
    #media-session.enable = true;
  };

  #Firefox not launching?
  #programs.firefox = {
  #  enable=true;
  #};

  environment.systemPackages = with pkgs; [ 
    cage
    #firefox #no h.265 support, #use module (below) instead to allow extra options
    #chromium #use module (below) to allow extra options
    wlr-randr

    tree
  ];

  #programs.chromium = {
  #  enable = true;
  #  #package = pkgs.chromium;
  #  
  #  extraOpts = {
  #    "BrowserSignin" = 0;
  #    "SyncDisabled" = true;
  #    "PasswordManagerEnabled" = false;
  #    "SpellcheckEnabled" = false;
  #  };
  #};

  programs.firefox = {
    enable = true;
    package = pkgs.firefox;
    preferences = {
      # Trying a smattering of preferences to get h.265 HEVC streams working
      "media.hevc.enabled" = true;
      #"media.ffmpeg.vaapi.enabled" = true;
      #media.webrtc.hw.h264.enabled = true; #This is the default
      #"media.rdd-vpx.enabled" = false;
      #"media.rdd-process.enabled" = true;
      #"widget.wayland-dmabuf-vaapi.enabled" = true;
      #"gfx.webrender.enabled" = true;
    };
  };

  #programs.chrome = {
  #  enable = true;
  #};

  services.cage = {
    enable=true;
    program="${pkgs.firefox}/bin/firefox --kiosk --private-window http://127.0.0.1:30125"; #Tried double dashes but it seemed to break firefox launch.
    #program="${pkgs.firefox}/bin/firefox --private-window http://127.0.0.1:30125"; #For development and debugging
    #program="${pkgs.firefox}/bin/firefox --kiosk --devtools --private-window http://127.0.0.1:30125"; #kiosk and devtools don't work together
    #
    #program="${pkgs.chromium}/bin/chromium --kiosk --noerrdialogs --no-first-run --no-default-browser-check http://127.0.0.1:30125"; #Does not launch in kiosk mode
    #program="${pkgs.google-chrome}/bin/google-chrome --kiosk --noerrdialogs --no-first-run --no-default-browser-check --incognito --disable-infobars http://127.0.0.1:30125";
    #
    user="kiosk";
  };

  

  #Not needed, running quadlets as root.
  #users.users.root.linger=true; #Allows execution of podman commands when su from another user. Also allows user-scoped quadlets to start on boot.

  # wait for network and DNS so cage service doesn't fail. Check with systemctl status cage-tty1
  systemd.services."cage-tty1"={
    requires = [
      "network-online.target"
      "network.target"
      #"systemd-resolved.service" #Does not seem to exist
      #"faux-show-backend.service"
    ];
    after = [
      "network-online.target"
      "network.target"
      "systemd-resolved.service"
      "faux-show-backend.service"
    ];
    # Gives network more time to connect to avoid a failed initial load of the dashboard
    preStart = ''
      sleep 5
    '';
    serviceConfig = {
    #  InaccessiblePaths = "/run/current-system/sw/share/icons"; #/usr/share/icons doesn't seem to exist on this nix
    #  Environment = "XCURSOR_PATH=${transparent-cursor}/Transparent";
      Environment = "GTK_THEME=Adwaita:dark";
      TimeoutStartSec = "60s";
      Restart = "always";
      RestartSec = "10s";
    };
  };

  #Allow kiosk without input variables
  #Allows boot but seems to bring mouse back
  #environment.variables = {
  #  WLR_LIBINPUT_NO_DEVICES=1;
  #};

  services.udev.extraRules = udev_extrarules_small_monitor;

  #Get dark theme
  #Neither works
  #dconf.settings = {
  #  "org/gnome/desktop/interface" = {
  #    color-scheme = "prefer-dark";
  #  };
  #};
  #gtk = {
  #  enable = true;
  #  theme = {
  #    name = "Adwaita-dark";
  #  };
  #};

  #Trying to figure out how to restart cage at command line
  #sudo systemctl restart cage-tty1 didn't work...

  networking.firewall.allowedTCPPorts = [
      #Should disable these to disallow external access, shouldn't be required for internal access but need to check this because of containerization of app
      #30125
      #30126
      443 
    ];
}