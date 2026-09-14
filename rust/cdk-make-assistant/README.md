# cdk-make-assistant

`cdk-make-assistant` discovers C-SKY CDK workspaces and projects, remembers a
build selection, and invokes `cdk-make.exe` without shell-specific quoting.

## Quick start

```powershell
cdk-make-assistant config set cdk-make "C:\Program Files\C-Sky\CDK\cdk-make.exe"
cdk-make-assistant configure D:\path\to\project
cdk-make-assistant build
cdk-make-assistant rebuild
cdk-make-assistant clean
```

Run `cdk-make-assistant inspect . --format json` to inspect the discovered
workspace and project data.

Project selections are stored in `.cdk-make-assistant.toml` at the project
root. The global executable configuration is created at
`%APPDATA%\cdk-make-assistant\config.toml` after an explicit `config set`.

Release archives contain `cdk-make-assistant.exe` and `install.ps1`. Run the
installer from PowerShell to copy the executable into the current user's
program directory and add that directory to the user `PATH`.

Native filesystem and XML implementations are used by default. Optional
Mike Farah `yq` v4 and `rg` executables can be enabled explicitly:

```powershell
cdk-make-assistant config set rg C:\path\to\rg.exe
cdk-make-assistant config set yq C:\path\to\yq.exe
```
