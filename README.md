<p align="center">
<img src="./docs/img/logo.png" alt="logo" width="380"/>
</p>

⭐ Zorka (Cyrillic: зорка, [ˈzorkə], star in Belarusian) is a fast, self contained and minimalistic url shortener, using [Actix](https://actix.rs/) to store and host the redirects.

You can read up on the usage of this tool in [docs/en/_readme.md](/docs/en/_readme.md).

In the case you just want to try this project out, get a prebuild docker image from [litvinav/zorka](https://hub.docker.com/r/litvinav/zorka) and run it with a exposed port 8080 or a port number specified via the PORT environment variable.

## Minimal example

```yaml
services:
  zorka:
    image: litvinav/zorka:v0.4.0
    ports:
    - 8080:8080
    volumes:
    - ./seed.csv:/app/seed.csv # first startup seeding; ignore for empty launch
    - backups:/app/backups     # internal backups on exit; ignore for emphemeral storage
    - ./configuration.yaml:/app/configuration.yaml # configuration override; ignore for default
    deploy:
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
        window: 120s
      # start with these resources and scale vertically, if required
      resources:
        limits:
          cpus: '0.15'
          memory: 30M
    cap_drop: [ALL]
volumes:
  backups: {}
```