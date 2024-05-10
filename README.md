This program adds itself to startup and runs the given binaries in the `config.toml` config file (on startup). It automatically sets the working directory as the binary's parent directory.

Default config:
```toml
[process.example]
path = 'C:\Example\Path\To\file.exe'
args = "-e -x -a -m -p -l -e"
hide = false

[process.another_example]
path = 'C:\Example2\Path\To\file.exe'
args = ""
hide = true
```
Setting the `hide` option to `true` will make the program to spawn the process with the `CREATE_NO_WINDOW` flag.

---
**Currently, only Windows is supported.**
