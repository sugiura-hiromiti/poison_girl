{ inputs, ... }: {
  imports = [ (inputs.files + "/flake-module.nix") ];
  perSystem = { lib, config, ... }: {
    files = {
      file = lib.mapAttrs' (
        name: drv: lib.nameValuePair ".github/workflows/${name}" { source = drv; }
      ) config.githubActions.workflowFiles;

    };
    apps = {
      sync-ci-workflow = {
        type = "app";
        program = lib.getExe config.files.writer.drv;
      };
    };
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
              name = "flake check (\${{ matrix.runner }})";
              permissions = {
                contents = "read";
                id-token = "write";
              };
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
