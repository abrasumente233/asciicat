{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        isDarwin = pkgs.lib.strings.hasSuffix "-darwin" system;
        rustToolchain = pkgs.rust-bin.stable.latest.default;
        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
        ] ++ lib.optional (isDarwin) darwin.apple_sdk.frameworks.SystemConfiguration;
        buildInputs = with pkgs; [ openssl sqlite ];
      in
      with pkgs;
      {
        devShells.default = mkShell {
          inherit buildInputs nativeBuildInputs;
        };
      });
}