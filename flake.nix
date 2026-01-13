{
  description = "Bitcoin Node Census";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        bitcoin-node-census = pkgs.rustPlatform.buildRustPackage rec {
          pname = "bitcoin-node-census";
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          # Copy static site files to share directory.
          postInstall = ''
            mkdir -p $out/share/site
            if [ -f site/index.html ]; then
              cp site/index.html $out/share/site/
            fi
            if [ -f site/favicon.svg ]; then
              cp site/favicon.svg $out/share/site/
            fi
            if [ -f site/census.js ]; then
              cp site/census.js $out/share/site/
            fi
          '';

          meta = with pkgs.lib; {
            description = "Track bitcoin node feature adoption";
            homepage = "https://github.com/nyonson/bitcoin-node-census";
            license = licenses.cc0;
            maintainers = [{
              name = "Nick Johnson";
              email = "nick@yonson.dev";
              github = "nyonson";
            }];
          };
        };
      in
      {
        packages = {
          default = bitcoin-node-census;
          bitcoin-node-census = bitcoin-node-census;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = bitcoin-node-census;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustup
            just
            python3
          ];
        };
      }
    ) // {
      nixosModules.default = { config, lib, pkgs, ... }:
        with lib;
        let
          cfg = config.services.bitcoin-node-census;
        in
        {
          options.services.bitcoin-node-census = {
            enable = mkEnableOption "Bitcoin Node Census service";

            package = mkOption {
              type = types.package;
              default = self.packages.${pkgs.stdenv.hostPlatform.system}.bitcoin-node-census;
              defaultText = literalExpression "pkgs.bitcoin-node-census";
              description = "The bitcoin-node-census package to use";
            };

            dataDir = mkOption {
              type = types.path;
              default = "/var/lib/bitcoin-node-census";
              description = "Directory to store census data and serve static site";
            };

            seedNode = {
              address = mkOption {
                type = types.str;
                description = "Bitcoin seed node address";
                example = "192.168.1.238";
              };

              port = mkOption {
                type = types.port;
                default = 8333;
                description = "Bitcoin seed node port";
              };
            };

            concurrent = mkOption {
              type = types.int;
              default = 32;
              description = "Maximum concurrent connections during census";
            };

            interval = mkOption {
              type = types.str;
              default = "weekly";
              description = "Systemd calendar expression for how often to run census";
              example = "hourly";
            };

            user = mkOption {
              type = types.str;
              default = "bitcoin-census";
              description = "User to run the service as";
            };

            group = mkOption {
              type = types.str;
              default = "bitcoin-census";
              description = "Group to run the service as";
            };

            web = {
              enable = mkEnableOption "web interface for census data" // {
                default = false;
              };

              assetsDir = mkOption {
                type = types.path;
                default = "${cfg.package}/share/site";
                defaultText = literalExpression "\${cfg.package}/share/site";
                description = "Path to web interface files to deploy.";
                example = literalExpression "./my-custom-site";
              };
            };

            backup = {
              enable = mkEnableOption "backup census data after each run" // {
                default = false;
              };

              destination = mkOption {
                type = types.str;
                default = "";
                description = "Backup destination (e.g., 'gemini2.lan:/var/backups/census/')";
                example = "gemini2.lan:/var/backups/census/";
              };

              command = mkOption {
                type = types.nullOr types.str;
                default = null;
                description = "Custom backup command. If null, uses scp with timestamped filename.";
                example = literalExpression ''"''${pkgs.rsync}/bin/rsync -a census.jsonl ''${cfg.backup.destination}"'';
              };
            };
          };

          config = mkIf cfg.enable {
            assertions = [
              {
                assertion = !cfg.backup.enable || cfg.backup.destination != "";
                message = "services.bitcoin-node-census.backup.destination must be set when backup.enable is true";
              }
            ];

            users.users.${cfg.user} = {
              isSystemUser = true;
              group = cfg.group;
              home = cfg.dataDir;
              createHome = true;
              homeMode = "0755";
            };

            users.groups.${cfg.group} = {};

            systemd.services.bitcoin-node-census = {
              description = "Bitcoin Node Census";
              after = [ "network.target" ];

              serviceConfig = {
                Type = "oneshot";
                User = cfg.user;
                Group = cfg.group;
                WorkingDirectory = cfg.dataDir;

                # Essential security.
                ProtectSystem = "strict";
                ProtectHome = true;
                ReadWritePaths = [ cfg.dataDir ];

                # Network restrictions.
                RestrictAddressFamilies = [ "AF_INET" "AF_INET6" ];

                # Basic hardening.
                NoNewPrivileges = true;
                PrivateTmp = true;
              };

              preStart = optionalString cfg.web.enable ''
                # Copy web interface files.
                cp -f ${cfg.web.assetsDir}/* .
              '';

              script = ''
                # Run census and append to census.jsonl.
                echo "Running census..."
                ${cfg.package}/bin/bitcoin-node-census run \
                  --address "${cfg.seedNode.address}" \
                  --port ${toString cfg.seedNode.port} \
                  --concurrent ${toString cfg.concurrent} \
                  --format jsonl \
                  >> census.jsonl

                echo "Census complete!"
              '' + optionalString cfg.backup.enable ''
                echo "Backing up census data..."
                ${if cfg.backup.command != null then
                  cfg.backup.command
                else
                  ''
                    TIMESTAMP=$(${pkgs.coreutils}/bin/date +%Y-%m-%d)
                    ${pkgs.openssh}/bin/scp census.jsonl "${cfg.backup.destination}census-$TIMESTAMP.jsonl"
                    echo "Backup complete: census-$TIMESTAMP.jsonl"
                  ''
                }
              '';
            };

            systemd.timers.bitcoin-node-census = {
              description = "Bitcoin Node Census Timer";
              wantedBy = [ "timers.target" ];

              timerConfig = {
                OnCalendar = cfg.interval;
                # Run if missed for whatever reason.
                Persistent = true;
                RandomizedDelaySec = "3h";
              };
            };
          };
        };
    };
}
