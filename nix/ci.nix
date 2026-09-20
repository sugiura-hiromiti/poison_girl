{ lib }: rec {
  checkout = {
    name = "Checkout";
    users = "actions/checkout@v4";
  };
  installNix = {
    name = "Install Nix";
    users = "cachix/install-nix-action@v24";
    with_ = {
      github_access_token = "\${{ github.token }}";
    };
  };
  cacheNix = {
    name = "Cache Nix store";
    users = "DeterminateSystems/magic-nix-cache-action@13";
  };
  setupNixSteps = [
    checkout
    installNix
    cacheNix
  ];
  mkFlakeCheckJob =
    {
      runsOn,
      timeoutMinutes ? 60,
    }:
    {
      name = "Nix flake check (${runsOn})";
      inherit runsOn timeoutMinutes;
      steps = setupNixSteps ++ [
        {
          name = "Nix flake check";
          run = "nix flake check -L --show-trace";
        }
      ];
    };
}
