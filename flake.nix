{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
        
        # 1. Define libraries needed for both compile-time linking and runtime loading
        buildDeps = with pkgs; [ 
          xorg.libX11 
          xorg.libXcursor 
          xorg.libXi 
          libxkbcommon 
        ];
        
        # 2. Define build tools
        nativeBuildDeps = with pkgs; [ pkg-config makeWrapper ];
      in
      {
        # 'nix build' output
        defaultPackage = naersk-lib.buildPackage {
          src = ./.;
          
          buildInputs = buildDeps;
          nativeBuildInputs = nativeBuildDeps;
          
          # This ensures the binary finds libxkbcommon when run outside the shell
          postInstall = ''
            wrapProgram $out/bin/spwn \
              --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath buildDeps}
          '';
        };

        # 'nix develop' / direnv output
        devShell = with pkgs; mkShell {
          buildInputs = [ cargo rustc rustfmt pre-commit rustPackages.clippy rust-analyzer ] ++ buildDeps;
          nativeBuildInputs = nativeBuildDeps;
          
          RUST_SRC_PATH = rustPlatform.rustLibSrc;

          # This ensures 'cargo run' finds libxkbcommon
          shellHook = ''
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath buildDeps}:$LD_LIBRARY_PATH
          '';
        };
      }
    );
}
