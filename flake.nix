{
  description = "A development environment for a Rust project with external libraries";
  
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs { inherit system; };
      
      overrides = builtins.fromTOML (builtins.readFile ./rust-toolchain.toml);
      libPath = with pkgs; lib.makeLibraryPath [
        mesa
        libGL
        libGLU
        # Load external libraries that you need in your rust project here
        pkgs.alsa-lib
        pkgs.wayland
        pkgs.wayland-protocols
        pkgs.wlroots
        pkgs.libxkbcommon
      ];
    in rec {
      devShell = pkgs.mkShell {
        buildInputs = with pkgs; [
          mesa
          libGL
          libGLU
          clang
          # Replace llvmPackages with llvmPackages_X, where X is the latest LLVM version (at the time of writing, 16)
          llvmPackages.bintools
          rustup
          alsa-lib
          pkg-config
          wayland
          wayland-protocols
          wlroots
          libxkbcommon
        ];
        RUSTC_VERSION = overrides.toolchain.channel;
        # https://github.com/rust-lang/rust-bindgen#environment-variables
        LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
        shellHook = ''
          export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
          export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
          export PKG_CONFIG_PATH=${pkgs.alsa-lib.dev}/lib/pkgconfig
        '';
        # Add precompiled library to rustc search path
        RUSTFLAGS = builtins.concatStringsSep " " (builtins.map (a: ''-L ${a}/lib'') [
          # Add libraries here (e.g. pkgs.libvmi)
        ]);
        LD_LIBRARY_PATH = libPath;
        # Add glibc, clang, glib, and other headers to bindgen search path
        BINDGEN_EXTRA_CLANG_ARGS =
        # Includes normal include path
        (builtins.concatStringsSep " " (builtins.map (a: ''-I"${a}/include"'') [
          # Add dev libraries here (e.g. pkgs.libvmi.dev)
          pkgs.glibc.dev
          pkgs.alsa-lib.dev
        ]))
        # Includes with special directory paths
        + " " + builtins.concatStringsSep " " [
          ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
          ''-I"${pkgs.glib.dev}/include/glib-2.0"''
          ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
        ];
      };
    });
}
