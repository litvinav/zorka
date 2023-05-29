<p align="center">
<img src="./docs/img/logo.jpeg" alt="logo" width="240"/>
</p>

⭐ Zorka (Cyrillic: зорка, [ˈzorkə], star in Belarusian) is a fast, self contained and minimalistic url shortener, using [Actix](https://actix.rs/) to store and host the redirects.

You can read up on the usage of this tool in [docs/en/_readme.md](/docs/en/_readme.md).

This project uses a custom backup mechanism with a custom .zorka file format. If you are interested in it, you can read more about it in [docs/en/zorka_files_and_backups.md](/docs/en/zorka_files_and_backups.md).

In the case you just want to try this project out, get a prebuild docker image from [litvinav/zorka](https://hub.docker.com/r/litvinav/zorka) and run it with a exposed port 8080.

## Minimal example

```yaml
services:
  zorka:
    image: litvinav/zorka:v0.3.0
    ports:
    - 8080:8080
    volumes:
    - ./seed.csv:/app/seed.csv # first startup seeding; ignore for empty launch
    - ./backups:/app/backups  # concurrent backups; ignore for emphemeral storage
    - ./backup.zorka:/app/backup.zorka # custom filetype for backup rotation; ignore for emphemeral storage
```