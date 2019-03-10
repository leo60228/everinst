{pkgs ? import <nixpkgs> {}}: with pkgs;
rustPlatform.buildRustPackage rec {
  name = "everinst";

  src = ./.;

  cargoSha256 = "0q68qyl2h6i0qsz82z840myxlnjay8p1w5z7hfyr8fqp7wgwa9cx";

  meta = with stdenv.lib; {
    description = "Everest installer.";
    platforms = platforms.all;
  };
}

