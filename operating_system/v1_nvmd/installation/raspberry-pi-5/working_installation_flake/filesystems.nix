{ config, lib, file_system_device, ... }:

let
  firmwarePartition = lib.recursiveUpdate {
    # label = "FIRMWARE";
    priority = 1;

    type = "0700";  # Microsoft basic data
    attributes = [
      0 # Required Partition
    ];

    size = "1024M";
    content = {
      type = "filesystem";
      format = "vfat";
      # mountpoint = "/boot/firmware";
      mountOptions = [
        "noatime"
        "noauto"
        "x-systemd.automount"
        "x-systemd.idle-timeout=1min"
      ];
    };
  };

  espPartition = lib.recursiveUpdate {
    # label = "ESP";

    type = "EF00";  # EFI System Partition (ESP)
    attributes = [
      2 # Legacy BIOS Bootable, for U-Boot to find extlinux config
    ];

    size = "1024M";
    content = {
      type = "filesystem";
      format = "vfat";
      # mountpoint = "/boot";
      mountOptions = [
        "noatime"
        "noauto"
        "x-systemd.automount"
        "x-systemd.idle-timeout=1min"
        "umask=0077"
      ];
    };
  };

in {

  # https://nixos.wiki/wiki/Btrfs#Scrubbing
  services.btrfs.autoScrub = {
    enable = true;
    interval = "monthly";
    fileSystems = [ "/" ];
  };

  fileSystems = {
    # mount early enough in the boot process so no logs will be lost
    "/var/log".neededForBoot = true;
  };

  disko.devices.disk.main = {
    type = "disk";
    device = file_system_device;

    content = {
      type = "gpt";
      partitions = {

        FIRMWARE = firmwarePartition {
          label = "FIRMWARE";
          content.mountpoint = "/boot/firmware";
        };

        ESP = espPartition {
          label = "ESP";
          content.mountpoint = "/boot";
        };

        system = {
          type = "8305";  # Linux ARM64 root (/)

          size = "100%";
          content = {
            type = "btrfs";
            extraArgs = [
              # "--label nixos"
              "-f"  # Override existing partition
            ];
            postCreateHook = let
              thisBtrfs = config.disko.devices.disk.main.content.partitions.system.content;
              device = thisBtrfs.device;
              subvolumes = thisBtrfs.subvolumes;

              makeBlankSnapshot = btrfsMntPoint: subvol: let
                subvolAbsPath = lib.strings.normalizePath "${btrfsMntPoint}/${subvol.name}";
                dst = "${subvolAbsPath}-blank";
                # NOTE: this one-liner has the same functionality (inspired by zfs hook)
                # btrfs subvolume list -s mnt/rootfs | grep -E ' rootfs-blank$' || btrfs subvolume snapshot -r mnt/rootfs mnt/rootfs-blank
              in ''
                if ! btrfs subvolume show "${dst}" > /dev/null 2>&1; then
                  btrfs subvolume snapshot -r "${subvolAbsPath}" "${dst}"
                fi
              '';
              # Mount top-level subvolume (/) with "subvol=/", without it 
              # the default subvolume will be mounted. They're the same in
              # this case, though. So "subvol=/" isn't really necessary
            in ''
              MNTPOINT=$(mktemp -d)
              mount ${device} "$MNTPOINT" -o subvol=/
              trap 'umount $MNTPOINT; rm -rf $MNTPOINT' EXIT
              ${makeBlankSnapshot "$MNTPOINT" subvolumes."/rootfs"}
            '';
            subvolumes = {
              "/rootfs" = {
                mountpoint = "/";
                mountOptions = [ "compress=zstd" "noatime" ];
              };
              "/nix" = {
                mountpoint = "/nix";
                mountOptions = [ "compress=zstd" "noatime" ];
              };
              "/home" = {
                mountpoint = "/home";
                mountOptions = [ "compress=zstd" "noatime" ];
              };
              "/log" = {
                mountpoint = "/var/log";
                mountOptions = [ "noatime" ];
              };
              #Tried this and got an error subsequently. File was made, but got
                ##Apr 25 15:20:55 rpi5 systemd[1]: Activating swap /.swapvol/swapfile...
                ##Apr 25 15:20:55 rpi5 swapon[5012]: swapon: /.swapvol/swapfile: swap format pagesize does not match.
                ##Apr 25 15:20:55 rpi5 swapon[5012]: swapon: /.swapvol/swapfile: reinitializing the swap.
                ##Apr 25 15:20:55 rpi5 swapon[5016]: swapon: failed to execute mkswap: No such file or directory
                ##Apr 25 15:20:55 rpi5 systemd[1]: \x2eswapvol-swapfile.swap: Swap process exited, code=exited, status=255/EXCEPTION
                ##Apr 25 15:20:55 rpi5 systemd[1]: \x2eswapvol-swapfile.swap: Failed with result 'exit-code'.
                ##Apr 25 15:20:55 rpi5 systemd[1]: Failed to activate swap /.swapvol/swapfile.
                ##Command 'ssh -o ControlMaster=auto -o ControlPath=/tmp/nixos-rebuild.g290y6h1/ssh-%n -o ControlPersist=60 root@10.10.10.160 -- env NIXOS_INSTALL_BOOTLOADER=0 systemd-run -E LOCALE_ARCHIVE -E NIXOS_INSTALL_BOOTLOADER --collect --no-ask-password --pipe --quiet --service-type=exec --unit=nixos-rebuild-switch-to-configuration /nix/store/g5cijvfhrc49i5wmsv2sifrlpb8pnrfh-nixos-system-rpi5-6.12.47-stable_20250916-kernel-raspberry-pi-5-25.11.20260407.4e92bbc/bin/switch-to-configuration switch' returned non-zero exit status 4.
              #"/swap" = {
              #  mountpoint = "/.swapvol";
              #  swap."swapfile" = {
              #    size = "2G";
              #    priority = 3; # (higher number -> higher priority)
              #    # to be used after zswap (set zramSwap.priority > this priority), 
              #    # but before "hibernation" swap
              #    # https://github.com/nix-community/disko/issues/651
              #  };
              #};
            };
          };
        };  # system

        swap = {
          type = "8200";  # Linux swap

          size = "2G";
          content = {
            type = "swap";
            resumeDevice = true;  # "hibernation" swap
            # zram's swap will be used first, and this one only 
            # used when the system is under pressure enough that zram and
            # "regular" swap above didn't work
            # https://github.com/systemd/systemd/issues/16708#issuecomment-1632592375
            # (set zramSwap.priority > btrfs' .swapvol priority > this priority)
            priority = 2;
          };
        };

      };
    };

  };  # disko.devices.disk.main
}