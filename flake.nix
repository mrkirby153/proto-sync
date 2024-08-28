{
  description = "A tool to manage out of repo protobufs";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
    in rec {
      packages = rec {
        proto-sync = pkgs.callPackage ./proto-sync.nix {};
        default = proto-sync;
      };
      devShell = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [protobuf lefthook openssl pkg-config];
        RUST_LOG = "debug";
        PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
      };
      formatter = pkgs.alejandra;
    });
}
