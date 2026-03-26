{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };
  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      pipewireLibPath = pkgs.lib.makeLibraryPath [
        pkgs.alsa-lib
        pkgs.alsa-plugins
        pkgs.pipewire
      ];
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [
          pkgs.rustc
          pkgs.cargo
          pkgs.alsa-lib
          pkgs.alsa-plugins
          pkgs.pipewire
          pkgs.pkg-config
        ];

        shellHook = ''
          export LD_LIBRARY_PATH="${pipewireLibPath}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
          export ALSA_PLUGIN_DIR="${pkgs.pipewire}/lib/alsa-lib:${pkgs.alsa-plugins}/lib/alsa-lib"
          export PIPEWIRE_MODULE_DIR="${pkgs.pipewire}/lib/pipewire-0.3"
          export SPA_PLUGIN_DIR="${pkgs.pipewire}/lib/spa-0.2"
        '';
      };
    };
}
