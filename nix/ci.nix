{
  perSystem = { ... }: {
    githubActions = {
      enable = true;
      workflows = {
        ci = {
          name = "ci";
          on = {
            push = {
              branches = [ "main" ];
            };
            pullRequest = {
              branches = [ "main" ];
            };
          };
          jobs = {
            permissions = {
              contents = "read";
              id-token = "write";
            };
            check = {
              name = "flake check (\${{ matrix.runner }})";
              strategy = {
                failFast = false;
                matrix = {
                  runner = [
                    "ubuntu-24.04-arm"
                    "macos-15"
                  ];
                };
              };
              runsOn = "\${{ matrix.runner }}";
              steps = [
                { uses = "actions/checkout@v4"; }
                { uses = "cachix/install-nix-action@v31"; }
                { uses = "DeterminateSystems/magic-nix-cache-action@v14"; }
                { run = "nix flake check -L --show-trace"; }
              ];
            };
          };
        };
      };
    };
  };
}
