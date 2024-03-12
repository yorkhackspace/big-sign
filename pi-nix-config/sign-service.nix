{
  lib,
  pkgs,
  config,
  ...
}:
with lib; let
  yhs-sign = (pkgs.callPackage ../yhs-sign.nix {}).default;
  cfg = config.services.yhs-sign;
in {
  options.services.yhs-sign = {
    enable = mkEnableOption "Big Sign";
  };

  config = mkIf cfg.enable {
    systemd.services.yhs-sign = {
      wantedBy = ["default.target"];
      serviceConfig.ExecStart = "${yhs-sign}/bin/yhs-sign";
    };
    environment.systemPackages = with pkgs; [
      yhs-sign
    ];
  };
}
