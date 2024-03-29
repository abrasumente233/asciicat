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

        # setting up crane lib with the shipyard registry
        # craneLibWithoutRegistry = crane.lib.${system}.overrideToolchain rustToolchain;
        # shipyardToken = builtins.readFile ./secrets/shipyard;
        # craneLib = craneLibWithoutRegistry.appendCrateRegistries [
        #   (craneLibWithoutRegistry.registryFromDownloadUrl {
        #     indexUrl = "ssh://git@ssh.shipyard.rs/abrasumente/crate-index.git";
        #     dl = "https://crates.shipyard.rs/api/v1/crates";
        #     fetchurlExtraArgs = {
        #       curlOptsList = [ "--header" "User-Agent: shipyard ${shipyardToken}" ];
        #     };
        #   })
        # ];
        craneLib = crane.lib.${system}.overrideToolchain rustToolchain;

        # common source and build dependencies
        isDarwin = pkgs.lib.strings.hasSuffix "-darwin" system;
        commonInputs = {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          nativeBuildInputs = with pkgs; [
            rustToolchain
            pkg-config
          ] ++ lib.optional (isDarwin) darwin.apple_sdk.frameworks.SystemConfiguration;
          buildInputs = with pkgs; [ openssl sqlite ];
        };

        # build the rust app
        bin = craneLib.buildPackage commonInputs // {
          cargoArtifacts = craneLib.buildDepsOnly commonInputs;
        };
      in
      with pkgs;
      {
        packages = {
          dockerImage = pkgs.dockerTools.buildImage {
            name = "asciicat";
            tag = "latest";
            config = {
              Cmd = [ "${bin}/bin/asciicat" ];
            };
          };
          default = bin;
        };
        devShells.default = mkShell {
          inputsFrom = [ bin ];
        };
      });
}
