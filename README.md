# Steam Achievement Unlocker

## Description

Allows unlocking or resetting achievements for Steam titles by App ID.
Works on Windows, Linux, and Mac, including apple silicon.
Make sure Steam is running and logged in.

> Now written in Rust, the .NET version lives in the `dotnet` branch

## Build / Run Instructions

```bash
cargo run -- --help
```

## Release

> Extract the release for your platform first

Run these commands in the terminal:

### Windows

```ps1
sau --help
```

### Mac / Linux

```bash
chmod +x sau
./sau --help
```

## Usage

Release
> sau [OPTIONS]

Source
> cargo run -- [OPTIONS]

| Option                        | Description                                                      |
|-------------------------------|------------------------------------------------------------------|
| `-i`, `--id <ID>`             | App ID (required)                                                |
| `-c`, `--clear`               | Clear achievements                                               |
| `-p`, `--parallel <PARALLEL>` | How many apps to process at once, too high will cause issues [1] |
| `-h`, `--help`                | Print help                                                       |
| `-V`, `--version`             | Print version                                                    |


You can combine arguments, for example `--id 123 --clear`

To find an App ID, search for the game on [SteamDB](https://steamdb.info/) or check the game's Steam store page URL.
