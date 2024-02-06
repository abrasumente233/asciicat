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
    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default;
        isDarwin = pkgs.lib.strings.hasSuffix "-darwin" system;
        craneLibWithoutRegistry = crane.lib.${system}.overrideToolchain rustToolchain;
        shipyardToken = builtins.readFile ./secrets/shipyard;
        craneLib = craneLibWithoutRegistry.appendCrateRegistries [
          (craneLibWithoutRegistry.registryFromDownloadUrl {
            indexUrl = "ssh://git@ssh.shipyard.rs/abrasumente/crate-index.git";
            dl = "https://crates.shipyard.rs/api/v1/crates";
            fetchurlExtraArgs = {
              curlOptsList = [ "--header" "User-Agent: shipyard ${shipyardToken}" ];
            };
          })
        ];
        commandArgs = {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          nativeBuildInputs = with pkgs; [
            rustToolchain
            pkg-config
          ] ++ lib.optional (isDarwin) darwin.apple_sdk.frameworks.SystemConfiguration;
          buildInputs = with pkgs; [ openssl sqlite ];
        };
        cargoArtifacts = craneLib.buildDepsOnly commandArgs;
        bin = craneLib.buildPackage commandArgs // {
          # TODO: commandArgs is stupid...
          inherit cargoArtifacts;
        };
        dockerImage = pkgs.dockerTools.buildImage {
          name = "asciicat";
          tag = "latest";
          copyToRoot = [ bin ];
          config = {
            # TODO: why prefix with ${bin}?
            Cmd = [ "${bin}/bin/asciicat" ];
          };
        };
      in
      with pkgs;
      {
        packages = {
          inherit bin dockerImage;
          default = bin;
        };
        devShells.default = mkShell {
          inputsFrom = [ bin ];
        };
      });
}
