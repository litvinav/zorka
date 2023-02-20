# Summary

Zorka (Cyrillic: зорка, [ˈzorkə], star in Belarusian) is a fast and powerful url shortener.
Zorka aims to be fast, self contained and minimalistic, using sqlite to store and actix to host the redirects.

## Usage

Zorka is supposed to be seeded on launch and serve redirects via the get route.
You can scale this url shortener to your requirements, since its hosting the same url's on all instances.

```sh
GET     /shortcut/:slug # fetches a url shortcut redirection
PUT     /shortcut       # put a new url shortcut entry during runtime
DELETE  /shortcut       # deletes entries during runtime by slug or url
GET     /health         # readiness and liveness health
```

Expose `/shortcut/...` to the internet and use `/shortcut` internally.
Also Zorka supports seeding on launch and imports slug and url data from a `./seed.csv`.

#### Examples

Compose:
```yaml
services:
  zorka:
    image: litvinav/zorka:latest
    environment:
      RUST_LOG: "zorka=info"
    ports:
    - 8080:8080
    volumes:
    - ./seed.csv:/app/seed.csv:ro
```
If you are feeling paranoid or cannot use a amd64 image, you can always build Zorka from source and store the image in your registry.

Seeding example: seed.csv
```yaml
yt,https://www.youtube.com/watch?v=dQw4w9WgXcQ
ya,https://yandex.ru/
gh,https://github.com
```
As you can see each line consists of the slug and a full url.
