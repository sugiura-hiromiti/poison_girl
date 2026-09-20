{
  perSystem = { ... }: {
    githubActions = {
      enable = true;
      workflows = {
        ci = {
          jobs = {
            check = {
              steps = [ { run = "nix flake check -L --show-trace"; } ];
            };
          };
        };
      };
    };
  };
}
