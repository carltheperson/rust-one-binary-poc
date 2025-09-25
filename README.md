# Rust-one-binary-POC

## Building the frontend

```bash
# In frontend/
npm i
npm run build
```

## Testing SSL using `pebble`

Get yourself `pebble` from https://github.com/letsencrypt/pebble

You should save `test/certs/pebble.minica.pem` in this repo location as `pebble.minica.pem` (to be brought into the binary).

You can now run:

```bash
# From inside pebble/ dir (that you cloned with git)
./pebble -config pebble-config.json -strict=false
```

Then you can simply do:

```bash
cargo run # Brings in ./pebble.minica.pem
```

Tip 1: I found it easier to use the `pebble` binary directly, but I know you can also use Docker.

Tip 2: Current `pebble` version gives you an error (`error: Order(Acme(Json(Error("unknown variant dns-account-01, expected one of http-01, dns-01, tls-alpn-01", line: 27, column: 33))))`) with `rustls-acme`. I used pebble `v2.5.1` to get around this.


### Make Firefox trust this stuff

Get the cert from [https://127.0.0.1:15000/roots/0](https://127.0.0.1:15000/roots/0). 

Convert to `.crt`:

```bash
openssl x509 -in FILE_IN -out FILE_OUT
```

Go to Settings > Privacy & Security > Certificates > View Certificates > Import, then select and add your certificate.

