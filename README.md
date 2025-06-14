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

Data published at [census.yonson.dev](https://census.yonson.dev/).

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
    
    # Seed node configuration
    seedNode = {
      address = "seed.bitcoin.sipa.be";
      port = 8333;
    };
    
    # Performance tuning
    concurrent = 32;
    
    # Schedule (systemd calendar format)
    interval = "weekly";  # or "daily", "hourly", "monthly", etc.
    
    # Data storage
    dataDir = "/var/lib/bitcoin-node-census";
    
    # Optional: GCP publishing
    gcp = {
      bucket = "gs://my-census-bucket";
      credentialsFile = "/run/secrets/gcp-service-account.json";
    };
  };
}
```

The service is designed to run automatically on the configured schedule, but you can test it manually.

```bash
# Start a census run (non-blocking)
sudo systemctl start --no-block bitcoin-node-census

# Monitor progress in real-time
sudo journalctl -u bitcoin-node-census -f

# Check service status
sudo systemctl status bitcoin-node-census

# View generated data
sudo ls -la /var/lib/bitcoin-node-census/site/
```

**Note**: Census operations can take several hours to complete as they crawl the entire bitcoin network. Use `--no-block` to avoid hanging your terminal session.

## Contributing

Guidelines are codified in the [justfile](justfile).

## License

The code in this project is licensed under [Creative Commons CC0 1.0](LICENSE).
