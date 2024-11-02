{
  lib,
  rustPlatform,
  openssl,
  pkg-config,
}:
rustPlatform.buildRustPackage {
  pname = "proto-sync";
  version = "0.1.0";
  src = ./.;
  cargoHash = "sha256-INBCnt5mKetTRIaq6SF4k7XUqlSq9qoEVy40MU0i9lE=";
  meta = {
    description = "A tool to manage out of repo protobufs";
    license = lib.licenses.mit;
  };
  nativeBuildInputs = [openssl pkg-config];
  PKG_CONFIG_PATH = "${openssl.dev}/lib/pkgconfig";
}
