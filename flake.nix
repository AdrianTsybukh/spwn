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
              --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath buildDeps} \
              --prefix XDG_DATA_DIRS : "$XDG_DATA_DIRS"
          '';
        };

        # 'nix develop' / direnv output
# 'nix develop' / direnv output
         devShell = with pkgs; 
           let
             rustToolchain = symlinkJoin {
               name = "rust-toolchain";
               paths = [ cargo rustc rust-analyzer rustfmt rustPackages.clippy ];
             };
           in
           mkShell {
             buildInputs = [ rustToolchain ] ++ buildDeps;
             nativeBuildInputs = nativeBuildDeps;

             RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";

             shellHook = ''
                 export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath buildDeps}:$LD_LIBRARY_PATH
                 export PATH="${rustToolchain}/bin:$PATH"
# This specific sub-path is often what the IDE needs to 'see' the std crates
                 export RUST_SRC_PATH="${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}"
                 '';
           };
      }
    );
}
