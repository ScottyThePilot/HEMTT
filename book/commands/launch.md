# hemtt launch

<pre><code>Launch Arma 3 with your mod and dependencies.

Usage: hemtt.exe launch [OPTIONS]

Options:
    <a href="#-e---executable">-e, --executable &lt;executable&gt;</a>
        Arma 3 executable to launch

    <a href="commands-dev.md#-b---binarize">-b, --binarize</a>
        Use BI's binarize on supported files

    <a href="commands-dev.md#-o---optional">-o, --optional &lt;optional&gt;</a>
        Include an optional addon folder

    <a href="commands-dev.md#-o---all-optionals">-O, --all-optionals</a>
        Include all optional addon folders

    <a href="commands.md#-t---threads">-t, --threads &lt;threads&gt;</a>
        Number of threads, defaults to # of CPUs

    <a href="commands.md#-v">-v...</a>
        Verbosity level

    -h, --help
        Print help information (use `-h` for a summary)
</code>
</pre>

`hemtt launch` is used to build and launch a dev version of your mod. It will run the [`hemtt dev`](commands-dev.md) command internally after a few checks, options are passed to the `dev` command.

## Configuration

`hemtt launch` requires the [`mainprefix`](configuration.md#main-prefix) option to be set.

**.hemtt/project.toml**

```toml
mainprefix = "z"

[hemtt.launch]
workshop = [
    "450814997", # CBA_A3's Workshop ID
]
parameters = [
    "-skipIntro",           # These parameters are passed to the Arma 3 executable
    "-noSplash",            # They do not need to be added to your list
    "-showScriptErrors",    # You can add additional parameters here
    "-debug",
    "-filePatching",
]
executable = "arma3" # Default: "arma3_x64"
```


### workshop

A list of workshop IDs to launch with your mod. These are not subscribed to, and will need to be manually subscribed to in Steam.

### parameters

A list of [Startup Parameters](https://community.bistudio.com/wiki/Arma_3:_Startup_Parameters) to pass to the Arma 3 executable.

### executable

The name of the Arma 3 executable to launch. This is usually `arma3` or `arma3_x64`. Do not include the `.exe` extension, it will be added automatically on Windows. Only paths relative to the Arma 3 directory are supported.

## Options

### -e, --executable <executable>

The Arma 3 executable to launch. Overrides the `executable` option in the configuration file.

```bash
hemtt launch -e arma3profiling_x64 # Relative to the Arma 3 directory
hemtt launch -e "C:\Program Files\Steam\steamapps\common\Arma 3\arma3_x64.exe" # Absolute path
```
