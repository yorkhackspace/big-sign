{
  config,
  lib,
  pkgs,
  yhs-sign,
  ...
}: let
  ssh-key-baud = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIDtNdrz/Dwa6K0JL2mrReneEpBo4W6P0Hx8sEX5yZ1+Q hi@sbaudlr.com";
in {
  imports = [
    # Include the results of the hardware scan.
    ./hardware-configuration.nix
    ./sign-service.nix
  ];

  nix.settings.experimental-features = ["nix-command" "flakes"];
  nix.settings.require-sigs = false;

  # Use the extlinux boot loader. (NixOS wants to enable GRUB by default)
  boot.loader.grub.enable = false;
  # Enables the generation of /boot/extlinux/extlinux.conf
  boot.loader.generic-extlinux-compatible.enable = true;

  networking.hostName = "sign";
  networking.wireless.enable = true;
  networking.wireless.environmentFile = "/run/secrets/wireless.env";
  networking.wireless.networks = {
    "YorkHackspace" = {
      psk = "@HACKSPACE_PSK@";
    };
  };

  hardware.enableRedistributableFirmware = true;

  # Set your time zone.
  time.timeZone = "Europe/London";

  # Select internationalisation properties.
  i18n.defaultLocale = "en_GB.UTF-8";
  console = {
    keyMap = "uk";
  };

  users.users.yhs = {
    isNormalUser = true;
    extraGroups = ["wheel"];
  };

  # SSH keys.
  users.users.yhs.openssh.authorizedKeys.keys = [
    ssh-key-baud
  ];
  users.users.root.openssh.authorizedKeys.keys = [
    ssh-key-baud
  ];

  environment.systemPackages = with pkgs; [
    nano
    vim
    htop
  ];

  # Enable the OpenSSH daemon.
  services.openssh.enable = true;

  # Enable Big Sign.
  services.yhs-sign.enable = true;

  services.avahi = {
    enable = true;
    nssmdns4 = true;
    publish = {
      enable = true;
      addresses = true;
      domain = true;
      hinfo = true;
      userServices = true;
      workstation = true;
    };
};

  # Firewall configuration.
  networking.firewall.allowedTCPPorts = [22];
  networking.firewall.allowedUDPPorts = [];
  networking.firewall.enable = true;

  # This option defines the first version of NixOS you have installed on this particular machine,
  # and is used to maintain compatibility with application data (e.g. databases) created on older NixOS versions.
  #
  # Most users should NEVER change this value after the initial install, for any reason,
  # even if you've upgraded your system to a new NixOS release.
  #
  # This value does NOT affect the Nixpkgs version your packages and OS are pulled from,
  # so changing it will NOT upgrade your system.
  #
  # This value being lower than the current NixOS release does NOT mean your system is
  # out of date, out of support, or vulnerable.
  #
  # Do NOT change this value unless you have manually inspected all the changes it would make to your configuration,
  # and migrated your data accordingly.
  #
  # For more information, see `man configuration.nix` or https://nixos.org/manual/nixos/stable/options#opt-system.stateVersion .
  system.stateVersion = "23.11"; # Did you read the comment?
}
