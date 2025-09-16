# Steam Achievement Unlocker

## Description

Allows unlocking or resetting every achievement for owned Steam titles.
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

```
-i, --id <ID>              App ID
-c, --clear                Clear achievements
-o, --owned                All owned apps
-a, --all                  All known apps
-p, --parallel <PARALLEL>  How many apps to process at once, too high will cause issues [default: 1]
-h, --help                 Print help
-V, --version              Print version
```

You can combine arguments, for example --owned --clear

If you have family sharing enabled, you can use --all
