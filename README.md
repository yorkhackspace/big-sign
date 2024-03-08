# YHS Sign

This is a service for communicating with the YHS sign via a HTTP API.

For development, you can print the commands that will be send to the sign by passing the flag `--fake-serial`, this will log the command buffers at the `trace` log level (setting the environment variable `RUST_LOG=yhs_sign=trace` will let you see these messages). In this mode, no communication will be made with the sign.

## HTTP Methods

###  `PUT /text/:textKey`
e.g. `PUT /text/test`
Writes some text to the sign immediately. Supported keys are hard-coded for now (test, lulzbot, anycubic).

The request body should be:
```json
{
    "text": "Some awesome text to write to the sign"
}
```

### `POST /script` 
Executes a script, allowing multiple messages to be written to the sign in a single request.
Currently, the only supported language is [Rhai](https://rhai.rs/) but PRs to add more languages would be awesome!

The request body:
```json
{
    "language": "rhai",
    "script": "write(\"Hello World!\");delay_seconds(1);write(\"Trans Rights!\");"
}
```

## Deploying

From a system running NixOS with appropriate SSH keys set up you can run something like:

`nixos-rebuild switch --target-host root@yhs-sign --flake .#yhs-sign`

## Things that need doing (Just a brain-dump)
- Make the sign rotate through all messages that have been sent to it.
- Make the scripting do things like
  - Write text to a file and recall later for speedier text drawing e.g. `let text = store_text("Hello"); load_text(text);`.
- Add more languages for scripting (BASIC, anyone?)
- Graphics!
- Other flashy things!
- ~~re-write it in [insert language of choice here]~~
- Make the API return useful errors.
- Maybe expose a socket API for folks who like that sort of thing.
