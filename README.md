# Boson ⚛️

Boson is a Steam compatibility tool that allows you to run Electron-based games with a native build of Electron,
rather than using the game's bundled Electron version and running it thorugh Proton.
Think of it like [Boxtron], [Roberta], or [Luxtorpeda], but for Electron games.

Please check [this list](https://steamdb.info/tech/Container/Electron/) to see if you need/want to use Boson for your game.

[Boxtron]: https://github.com/dreamer/boxtron
[Roberta]: https://github.com/dreamer/roberta
[Luxtorpeda]: https://github.com/dreamer/luxtorpeda

Inspired by [NativeCookie](https://github.com/Kesefon/NativeCookie/).

Currently, only [Cookie Clicker](https://orteil.dashnet.org/cookieclicker/) is supported at the moment.

## How does it work?

Boson is a Steam Play compatibility tool, intercepting calls to run the game from its own executable, and redirects
the executable to a native Electron executable, running the game using the provided Electron build instead.

## Why Boson?

While some games are developed essentially as Electron PWAs, some game developers still refuse to publish native ports of their games.
Expecting users to simply either run the game on Windows, or run its Electron executable under Proton.

While this is a quick-and-dirty way to simply run Electron games on Linux, sometimes it may cause some issues due to the fact that we're running Chromium
inside Proton, even though Electron has a native Linux build, e.g:

- Graphical artifacts
- Missing fonts (because the fonts are loaded exclusively from the Proton prefix)
- Scaling issues
- Other general compatibility issues

Boson works around this issue entirely by simply just running the game using a native Electron
build rather than running Electron inside Steam Proton.

## Usage

1. Install Electron from your package manager, or download the binaries from the [Electron website](https://www.electronjs.org/) (see note below)
2. Download the latest release tarball
3. Extract to `~/.steam/root/compatibilitytools.d/`. You should have a directory structure like this:
   ```
   ~/.steam/root/compatibilitytools.d/boson/
   ├── boson
   ├── toolmanifest.vdf
   └── compatibiltytool.vdf
   ```
4. Restart or start Steam if you haven't already
5. Right-click on the game you want to run with Boson, and select `Properties > Force the use of a specific Steam Play compatibility tool > Boson`
6. Run the game, And that's it! The game should now be running using the native Electron build.

## Building

All you need to build Boson is a Rust toolchain, and the `cargo` build tool. That's it.

Install Rust by using [rustup](https://rustup.rs/), and then run the following commands:

```sh
make
```

The resulting Steam compatibility tool will be outputted to `build/`. You can just copy the resulting files to `~/.steam/root/compatibilitytools.d/` and you're good to go.

### Notes

- If you're using an electron binary that isn't in your $PATH and called `electron`, you can set the `ELECTRON_PATH` environment variable in your Steam launch options to point to the electron binary you want to use, e.g:
  ```
  ELECTRON_PATH=/path/to/electron %command%
  ```
- Due to some incompatibility issues with the Steam overlay, it's recommended to disable the Steam overlay for the game you're running with Boson. Boson is currently hardcoded to remove any `LD_PRELOAD` envars on runtime, to prevent the Steam overlay from being loaded.
- If you're trying to run some other Electron-based game that isn't Cookie Clicker that doesn't have the game path set to `resources/app`, you can set the `BOSON_LOAD_PATH` environment variable in your Steam launch options to point to the game's resource path (a plain folder or an `app.asar` file), e.g:
  ```
  BOSON_LOAD_PATH=/path/to/game/resources/app %command%
  ```
  Note that paths here are relative to the game's installation directory.

#### Running Cookie Clicker (and other Greenworks games) with Boson

This guide assumes you already bought Cookie Clicker on Steam, and have it installed.

It also assumes that your CPU architecture is x86_64, and you're running a 64-bit Linux distribution, Steam for Linux only supports x86_64 for now.

If you'd like to play the web version, just go to the [Cookie Clicker website](https://orteil.dashnet.org/cookieclicker/).
The only differences between the web and Steam version is that the Steam version has cloud saves, Steam achievements, Workshop support, and an OST by C418 (Yes, the Minecraft guy).

To get the Steamworks API to work with Cookie Clicker, you need to do the following:

1. Downloads the Steamworks SDK from the [Steamworks website](https://partner.steamgames.com/downloads/list)
2. Take note of these files from the SDK, we will move this to Greenwork's library location:
   - `sdk/redistributable_bin/linux64/libsteam_api.so`
   - `sdk/public/steam/lib/linux64/libsdkencryptedappticket.so`
3. Download the nightly builds of Greenworks for the respective compatible version of Electron from [here](https://greenworks-prebuilds.armaldio.xyz/), rename the resulting `.node` binary to `greenworks-linux64.node`
4. Once you downloaded the SDK, extract the SDK libraries to Cookie Clicker's installation directory, like this:
   ```
   ~/.local/share/Steam/steamapps/common/Cookie Clicker/
    ├── resources
        |── app
            |── greenworks
                |── lib
                    |──greenworks-linux64.node
                    |──libsteam_api.so
                    |──libsdkencryptedappticket.so
                    |──(*libraries from other platforms*)
   ```
5. Once you're done installing Greenworks, your copy of Cookie Clicker should now integrate with Steamworks, and you can now get achievements, cloud saves, and Workshop support as if you're still running the game on Windows, with the added benefit of Native Linux support (and Discord Rich Presence support) :3
