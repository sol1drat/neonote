let
  rustOverlay = import (fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rustOverlay ]; };

  mingw = pkgs.pkgsCross.mingwW64;
  mingwGcc = mingw.buildPackages.gcc;

  rustToolchain = pkgs.rust-bin.stable.latest.default.override {
    targets = [ "x86_64-pc-windows-gnu" ];
  };
in
pkgs.mkShell {
  nativeBuildInputs = [
    mingwGcc
    rustToolchain
  ];
  buildInputs = [ mingw.windows.pthreads ];

  CC_x86_64_pc_windows_gnu = "${mingwGcc}/bin/x86_64-w64-mingw32-gcc";
}
