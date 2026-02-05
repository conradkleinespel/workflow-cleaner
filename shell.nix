{pkgs ? import <nixpkgs> {}}: let
  code = pkgs.writeShellApplication {
    name = "code";
    text = ''
      nohup idea nosplash . &>/dev/null & disown
    '';
  };
in
  pkgs.mkShell {
    buildInputs = with pkgs; [
      rustup
      rustPlatform.rustLibSrc
      pkg-config
      openssl.dev
      code
    ];

    shellHook = ''
      export RUST_SRC_PATH="${pkgs.rustPlatform.rustLibSrc}"
    '';
  }
