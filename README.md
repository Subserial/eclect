# Eclect

Reports your Last.fm Now Playing to Discord using Rich Presence.

You must have Discord open on the same computer to use this application.

## How To Use

```
Usage: eclect [OPTIONS]

Options:
  -w, --workdir <WORKDIR>
          Persistent storage location (Last.fm session token) [default: /home/sb/.local/share/eclect]
  -q, --query-interval <QUERY_INTERVAL>
          Seconds between Last.fm queries for now playing [default: 15]
      --discord-app-id <DISCORD_APP_ID>
          The Discord app ID to use. Required unless --discord-app-id-file is specified
      --discord-app-id-file <DISCORD_APP_ID_FILE>
          A file containing the Discord app ID to use. Required unless --discord-app-id is specified
      --lastfm-api-key <LASTFM_API_KEY>
          The Last.fm API key to use. Required unless --lastfm--api-key-file is specified
      --lastfm-api-key-file <LASTFM_API_KEY_FILE>
          A file containing the Last.fm API key to use. Required unless --lastfm--api-key is specified
      --lastfm-secret <LASTFM_SECRET>
          The Last.fm API secret to use. Required unless --lastfm-secret-file is specified
      --lastfm-secret-file <LASTFM_SECRET_FILE>
          A file containing the Last.fm API secret to use. Required unless --lastfm-secret is specified
  -c, --config-file <FILE>
          Read flags from a TOML file. Exclusive to other arguments.
  -h, --help
          Print help (see a summary with '-h')
  -V, --version
          Print version
```

The application requires 3 different tokens:
- A Discord app ID, from Discord's
  [developer dashboard](https://discord.com/developers/applications)
- A Last.fm API key, from Last.fm's
  [API account request form](https://www.last.fm/api)
- The Last.fm API secret associated with the prior key

On first run, the program will store a token and return an authorization URL.
If you accept the permission request at the URL, the program will be able to
run.

## Developer Notes

The Last.fm endpoints are declared in Protobuf files under `proto/`. These are
rough guesses about the real endpoints.

The intermediate API for Last.fm is generated using
[a fork](https://github.com/Subserial/prost/tree/extensions) of
[a fork](https://github.com/nswarm/prost/tree/extensions) of
[PROST!](https://github.com/tokio-rs/prost).  This fork exposes extension
information to the service generator, allowing me to store method information
in the service definitions. If I didn't do this, the project would have been
completed months in advance.