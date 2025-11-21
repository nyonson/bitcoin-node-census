<div align="center">
  <h1>Bitcoin Node Census</h1>
  <p>
    <strong>Track node feature adoption on the bitcoin network</strong>
  </p>

  <p>
    <a href="https://github.com/nyonson/bitcoin-node-census/blob/master/LICENSE"><img alt="CC0 1.0 Universal Licensed" src="https://img.shields.io/badge/license-CC0--1.0-blue.svg"/></a>
    <a href="https://github.com/nyonson/bitcoin-node-census/actions?query=workflow%3ACI"><img alt="CI Status" src="https://github.com/nyonson/bitcoin-node-census/actions/workflows/ci.yml/badge.svg"/></a>
  </p>
</div>

## About

Crawl the bitcoin network and aggregate stats for easy tracking.

Data published at [census.yonson.dev](https://census.yonson.dev/) and [census.labs.yonson.dev](https://census.labs.yonson.dev/). The schema for the raw data is in [docs/census.schema.json](docs/census.schema.json).

## Usage

This project includes a NixOS module for running automated censuses on a schedule.

Add to your `flake.nix`:

```nix
{
  inputs = {
    bitcoin-node-census.url = "github:nyonson/bitcoin-node-census";
  };

  outputs = { self, nixpkgs, bitcoin-node-census }:
    nixpkgs.lib.nixosConfigurations.yourhost = nixpkgs.lib.nixosSystem {
      modules = [
        bitcoin-node-census.nixosModules.default
        # ... your other modules
      ];
    };
}
```

And configure it:

```nix
{
  services.bitcoin-node-census = {
    enable = true;

    seedNode = {
      address = "192.168.1.238";
      port = 8333;
    };
    concurrent = 32;
    interval = "weekly";  # or "daily", "hourly", "monthly", etc.
    dataDir = "/var/lib/bitcoin-node-census";
  };
}
```

The service is designed to run automatically on the configured schedule, but you can test it manually.

```bash
# Start a census run (non-blocking).
sudo systemctl start --no-block bitcoin-node-census
# Check service status.
sudo systemctl status bitcoin-node-census
```

**Note**: Census operations can take several hours to complete as they crawl the entire bitcoin network. Use `--no-block` to avoid hanging your terminal session.

## Contributing

Guidelines are codified in the [justfile](justfile).

## License

The code in this project is licensed under [Creative Commons CC0 1.0](LICENSE).
