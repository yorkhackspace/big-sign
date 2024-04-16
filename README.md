# YHS Sign

This is a service for communicating with the YHS sign via a HTTP API.

For development, you can set the serial port and baud rate with `--port` and  `--baudrate`. I recommend using socat to create a pty for testing like this: `socat -d -d -d pty,raw pty,raw` works on mac (although requires baudrate to be set to 0)

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

###  `GET /text/get/:label`
e.g. `GET /text/get/A`

Gets text from a given label from the sign. The response will be blank if the label does not contain text or is uninitialised.

The response body should be:
```json
{
    "text": "Text from the sign's memory"
}
```


## Building

the backend is built the normal rust way with `cargo build`, if you want to crossbuild for the pi grab the aarch64-unknown-linux-gnu gcc toolchain and run `cargo build  --target aarch64-unknown-linux-gnu`.

the frontend is built using vite. To build after you clone it:

```
npm install
npm run build
```

you should only have to run `npm install` once.

## Deploying

either use deploy.sh if you are on a unix-like and within the hackspace, or:

* Cross compile for aarch64-unknown-linux-gnu
* stop the big-sign service on the sign pi, 
* copy target/aarch64-unknown-linux-gnu/debug/yhs-sign to ~ on the sign pi
* copy whatever static content stuff has changed to the sign pi
* log in and restart the systemd service for big-sign.


## Things that need doing (Just a brain-dump)
- Make the sign rotate through all messages that have been sent to it.
- Graphics!
- Other flashy things!
- ~~re-write it in [insert language of choice here]~~
- Make the API return useful errors.
- Maybe expose a socket API for folks who like that sort of thing.
- sync clocks with the sign on startup
- expose main message list in the webUI to edit
