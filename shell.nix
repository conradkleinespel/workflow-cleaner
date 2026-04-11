{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
  buildInputs = with pkgs; [
    rustup
    pkg-config
    openssl.dev
  ];

  shellHook = ''
    rustup default stable
    rustup component add rust-src
  '';
}
