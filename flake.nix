{
	description = "My NES Emulator";

	inputs = {
		nixpkgs.url = "github:NixOS/nixpkgs/nixos-20.09";
		utils.url = "github:numtide/flake-utils";
		rust-overlay.url = "github:oxalica/rust-overlay";
		naersk.url = "github:nmattia/naersk";
	};

	outputs = { nixpkgs, rust-overlay, utils, naersk, ... }:
		utils.lib.eachDefaultSystem (system:
			let
				pkgs = import nixpkgs {
					inherit system;
					overlays = [ rust-overlay.overlay ];
				};
				rust = pkgs.rust-bin.nightly."2021-01-31".rust;
				naersk-lib = naersk.lib.x86_64-linux.override {
					cargo = rust;
					rustc = rust;
				};
			in
			rec {
				packages.mynes = naersk-lib.buildPackage {
					pname = "mynes";
					root = ./.;
					buildInputs = [ pkgs.x11 pkgs.libxkbcommon ];
					nativeBuildInputs = [ pkgs.pkg-config ];
				};

				defaultPackage = packages.mynes;

				apps.mynes = utils.lib.mkApp {
					drv = defaultPackage;
				};

				defaultApp = apps.mynes;

				devShell = pkgs.mkShell {
					nativeBuildInputs = [ rust pkgs.pkg-config ];
					buildInputs = [ pkgs.x11 pkgs.xkbcommon ];
				};
			}
		);
}