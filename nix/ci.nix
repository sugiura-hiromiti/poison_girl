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
            check = {
              runsOn = "ubuntu-latest";
              steps = [
                { uses = "actions/checkout@v4"; }
                { uses = "cachix/install-nix-action@v24"; }
                { run = "nix flake check -L --show-trace"; }
              ];
            };
          };
        };
      };
    };
  };
}
