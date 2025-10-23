# Boson ⚛️

Boson is a Steam compatibility tool that allows you to run certain games in their respective native runtimes,
bypassing Proton/Wine entirely for better compatibility and performance on Linux systems.
Think of it like [Boxtron], [Roberta], or [Luxtorpeda], but for certain games built
in cross-platform bytecode runtimes like JavaScript (Electron) and Lua (Love2D).

Inspired by [NativeCookie](https://github.com/Kesefon/NativeCookie/).

This tool adds better compatibility for some LOVE and Electron-based games on Linux, including but not limited to:

- [Cookie Clicker](https://store.steampowered.com/app/1454400/Cookie_Clicker)
- [We Become What We Behold](https://store.steampowered.com/app/1103210/We_Become_What_We_Behold_FanMade_Port)
- [natsuno-kanata](https://store.steampowered.com/app/1684660/natsunokanata__beyond_the_summer)
- [Balatro](https://store.steampowered.com/app/2379780/Balatro)

## Planned features

- [x] Database of tweaks for each supported game
    - [ ] Automatically set up Steamworks API for games that use it (see note above)
- [ ] Download and install Electron builds from the Electron website (currently you need to bring your own Electron binary)
- [x] TOML configuration file(s) for custom tweaks
- [ ] GUI for managing Boson (and displaying error messages)

## How does it work?

Boson is a Steam Play compatibility tool, intercepting the Steam Runtime to run the game from its own executable, and redirects
the executable to a compatible native runtime installed on the host system

## Why Boson?

While some games are developed essentially as cross-platform binaries, some game developers still refuse to publish Steam builds of their games for Linux and macOS,
Expecting users to simply either run the game on Windows, or run its Windows-only executable under Proton.

While this is a quick-and-dirty way to simply run these games on Linux, sometimes it may cause some issues due to Wine-Linux translation layers, when these games' runtimes already have native Linux support:

- Graphical artifacts
- Missing fonts (because the fonts are loaded exclusively from the Proton prefix)
- Scaling issues
- Other general compatibility issues

Boson works around this issue entirely by simply just running the game using a native Electron
build rather than running Electron inside Steam Proton.

## Usage

1. Install Electron from your package manager, or download the binaries from the [Electron website](https://www.electronjs.org/) (see note below), make sure the `electron` binary is in your `$PATH`.
2. Download the latest release tarball
3. Extract to `~/.steam/root/compatibilitytools.d/`. You should have a directory structure like this:

    ```sh
    ~/.steam/root/compatibilitytools.d/boson/
    ├── boson
    ├── toolmanifest.vdf
    └── compatibiltytool.vdf
    ```

4. Restart or start Steam if you haven't already
5. Right-click on the game you want to run with Boson, and select `Properties > Force the use of a specific Steam Play compatibility tool > Boson`
6. Run the game, And that's it! The game should now be running using Boson.

## Boson package contents

while Boson can be built entirely from source and used without any additional dependencies, the release tarballs contain some additional runtime libraries to improve compatibility with certain games, including:

- Steamworks API libraries for better compatibility with games that use Steamworks SDK
- luasteam, a Lua binding for Steamworks SDK, for better compatibility with Love2D-based games that use Steamworks SDK
- Greenworks, a Node.js binding for Steamworks SDK, for better compatibility with Electron-based games that use Steamworks SDK
- Steamworks.js, a JavaScript binding for Steamworks SDK, for better compatibility with Electron-based games that use Steamworks SDK

## Building

You require Rust and Node.js + NPM to build Boson.

Install Rust by using [rustup](https://rustup.rs/), and then run the following commands:

```sh
make
```

The resulting Steam compatibility tool will be outputted to `build/`. You can just copy the resulting files to `~/.steam/root/compatibilitytools.d/` and you're good to go.

## Notes

- If you're using an electron binary that isn't in your $PATH and called `electron`, you can set the `ELECTRON_PATH` environment variable in your Steam launch options to point to the electron binary you want to use, e.g:

    ```sh
    ELECTRON_PATH=/path/to/electron %command%
    ```

- Boson has a list of known ASAR paths that it checks through to find the game's files, if Boson somehow cannot find the electron ASAR assets path, you can set a custom environment variable to tell Boson where to find the game data, e.g:

    ```sh
    BOSON_LOAD_PATH=/path/to/asar %command%
    ```

### Running Cookie Clicker (and other Greenworks games) with Boson

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

    ```txt
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

### Case 2: Running Balatro with Boson

Balatro is a Love2D-based game that can be run with Boson as well.

You will be required to install Love2D on your system and add it to your `$PATH` as `love` for this to work.

After installing Boson as a Steam compatibility tool and setting it for Balatro, the game will simply work out of the box without any additional steps.

To enable support for Steamworks features like achievements and cloud saves, you will need to install [Lovely Injector](https://github.com/ethangreen-dev/lovely-injector) and [SMODS](https://github.com/Steamodded/smods)

For Lovely Injector, if you're on Fedora and have Terra enabled, you can install it to the system path using:

```sh
sudo dnf install lovely-injector
```

For other distributions, you may either compile it from source, or download the prebuilt binaries from the [releases page](https://github.com/ethangreen-dev/lovely-injector/releases), and place the `liblovely.so` file to `~/.local/share/Steam/compatibilitytools.d/boson/lib/`, or any of the directories in your `LD_LIBRARY_PATH`.

The default Lovely Injector mods directory is `~/.config/love/Mods`, you can clone the SMODS repository or download a tagged release and extract it there.

You may also change the mods directory by setting the `LOVELY_MOD_DIR` environment variable in your Steam launch options, e.g:

```sh
LOVELY_MODS_DIR=/path/to/Mods %command%
```

Boson will not come included with Lovely Injector as the library may be updated frequently, and including it may cause compatibility issues.

> [!NOTE]
> The default Balatro preset config launches the game with the `--fused` global LOVE option, which tells LOVE that the game is packaged as a single fused `.love` bundle. This also changes the game's save directory to `~/.local/share/Balatro` instead of the usual `~/.local/share/love/Balatro`. See the [LOVE documentation](https://love2d-community.github.io/love-api/#filesystem_isFused) for more information.
>
> You may also want to symlink this to to `/home/cappy/.local/share/Steam/steamapps/compatdata/2379780/pfx/drive_c/users/steamuser/AppData/Roaming/Balatro` for compatibility with Steam Cloud saves for syncing saves between devices.

## In this space

- [Steam Tinker Launch](https://github.com/sonic2kk/steamtinkerlaunch) A similar Steam compatibility tool manager with a GUI, written as a large Bash script with a Zenity-based GUI. Only supports some runtimes like Boxtron and Roberta.
- [Roberta](https://github.com/dreamer/roberta) Steam compatibility tool for running ScummVM-based games with native ScummVM builds
- [Boxtron](https://github.com/dreamer/boxtron) Steam compatibility tool for running DOSBox-based games with native DOSBox builds
- [Luxtorpeda](https://github.com/luxtorpeda-dev/luxtorpeda) Steam compatibility tool for running various games with their respective native Linux engine/source ports, closest alternative to Boson in the same domain.
