# wasabi

a custom configuration tool for Windows Sandbox

[![Image from Gyazo](https://i.gyazo.com/517248a42e90429f5293e725edab0d68.gif)](https://gyazo.com/517248a42e90429f5293e725edab0d68)

## usage

1. generate TOML
```cmd
> wasabi generate sandbox.toml --shared-dir ./tools --shared-writable-dir ./logs
```

* `--disable-network`
    * disable network in sandbox

* `--disable-vgpu`
    * disable vGPU in sandbox

* `--shared-dir`
    * readonly shared dir
        * relative path
        * multiple path (`--shared-dir ./share1 ./share2 ...`)
    
* `--shared-writable-dir`
    * r/w shared dir
        * relative path
        * multiple path (`--shared-writable-dir ./writable1 ./writable2 ...`)
    
* `--logon-command`
    * init command
        * kick `init.bat` in` tools` shared directory, as default (also as a sample)

2. edit TOML
```cmd
> code sandbox.toml
```

3. generate WSB
```cmd
> wasabi build sandbox.toml -o sandbox.wsb
```

4. run wsb
```cmd
> sandbox.wsb
```

### generate WSB directly

add `-d` flag when generating

```cmd
> wasabi generate -d sandbox.wsb --shared-dir ./tools --shared-writable-dir ./logs
> sandbox.wsb
```
