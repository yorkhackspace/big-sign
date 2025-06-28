# YHS Sign

This is a service for communicating with the YHS sign via a HTTP API.

For development, you can set the serial port and baud rate with `--port` and  `--baudrate`. I recommend using socat to create a pty for testing like this: `socat -d -d -d pty,raw pty,raw` works on mac (although requires baudrate to be set to 0)

## HTTP Methods

###  `PUT /topics/:topicName`
e.g. `PUT /topics/test`
Writes some text to the sign immediately. And store it within the specified topic.

The request body should be:
```json
{
    "lines": [
        "First line of text",
        "Second line of text",
        "etc."
    ]
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
