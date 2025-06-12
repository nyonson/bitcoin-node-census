{
  description = "Bitcoin Node Census - Track bitcoin node feature adoption";

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
            google-cloud-sdk
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
              default = self.packages.${pkgs.system}.bitcoin-node-census;
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
                description = "Bitcoin seed node address (IP or hostname)";
                example = "seed.bitcoin.sipa.be";
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
            
            favicon = mkOption {
              type = types.nullOr types.path;
              default = null;
              description = "Path to custom favicon.svg file. If null, uses the default Bitcoin favicon from the package";
              example = "/path/to/custom-favicon.svg";
            };
            
            gcp = {
              bucket = mkOption {
                type = types.nullOr types.str;
                default = null;
                example = "gs://my-bitcoin-census-bucket";
                description = "GCP bucket to publish results to. If null, results are only stored locally";
              };
              
              credentialsFile = mkOption {
                type = types.nullOr types.path;
                default = null;
                example = "/run/secrets/gcp-service-account.json";
                description = "Path to GCP service account credentials JSON file";
              };
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
          };
          
          config = mkIf cfg.enable {
            users.users.${cfg.user} = {
              isSystemUser = true;
              group = cfg.group;
              home = cfg.dataDir;
              createHome = true;
            };
            
            users.groups.${cfg.group} = {};
            
            systemd.services.bitcoin-node-census = {
              description = "Bitcoin Node Census";
              after = [ "network.target" ];
              
              # Set the HOME env var to give the service
              # a home at the data directory, and also tack
              # on the GCP creds if set.
              environment = mkMerge [
                {
                  HOME = cfg.dataDir;
                }
                (mkIf (cfg.gcp.credentialsFile != null) {
                  GOOGLE_APPLICATION_CREDENTIALS = cfg.gcp.credentialsFile;
                })
              ];
              
              serviceConfig = {
                Type = "oneshot";
                User = cfg.user;
                Group = cfg.group;
                WorkingDirectory = cfg.dataDir;
                
                # Security hardening
                PrivateTmp = true;
                ProtectSystem = "strict";
                ProtectHome = true;
                ReadWritePaths = [ cfg.dataDir ];
                NoNewPrivileges = true;
                RestrictNamespaces = true;
                RestrictRealtime = true;
                RestrictSUIDSGID = true;
                LockPersonality = true;
                ProtectClock = true;
                ProtectHostname = true;
                ProtectKernelLogs = true;
                ProtectKernelModules = true;
                ProtectKernelTunables = true;
                ProtectControlGroups = true;
                RestrictAddressFamilies = [ "AF_INET" "AF_INET6" ];
                SystemCallFilter = [ "@system-service" "~@privileged" ];
              };
              
              preStart = ''
                # Ensure site directory exists.
                mkdir -p site
                
                # Copy static files from package if they don't exist.
                if [ ! -f site/index.html ]; then
                  if [ -f ${cfg.package}/share/site/index.html ]; then
                    cp ${cfg.package}/share/site/index.html site/
                  else
                    echo "Warning: No index.html found in package"
                  fi
                fi
                
                # Copy favicon - either custom or default from package.
                if [ ! -f site/favicon.svg ]; then
                  ${if cfg.favicon != null then ''
                    cp ${cfg.favicon} site/favicon.svg
                  '' else ''
                    if [ -f ${cfg.package}/share/site/favicon.svg ]; then
                      cp ${cfg.package}/share/site/favicon.svg site/
                    else
                      echo "Warning: No favicon.svg found in package"
                    fi
                  ''}
                fi
              '';
              
              script = ''
                # Run census and append to census.jsonl.
                echo "Running census..."
                ${cfg.package}/bin/bitcoin-node-census run \
                  --address "${cfg.seedNode.address}" \
                  --port ${toString cfg.seedNode.port} \
                  --concurrent ${toString cfg.concurrent} \
                  --format json \
                  >> site/census.jsonl
                
                # Publish to GCP if configured.
                ${optionalString (cfg.gcp.bucket != null) ''
                  echo "Publishing to GCP bucket ${cfg.gcp.bucket}..."
                  ${pkgs.google-cloud-sdk}/bin/gcloud storage rsync \
                    --recursive \
                    --delete-unmatched-destination-objects \
                    site/ ${cfg.gcp.bucket}/
                  
                  # Set content type for .jsonl file
                  ${pkgs.google-cloud-sdk}/bin/gcloud storage objects update \
                    ${cfg.gcp.bucket}/census.jsonl \
                    --content-type="application/x-ndjson" \
                    2>/dev/null || true
                  
                  echo "Publishing complete!"
                ''}
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
