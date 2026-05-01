# Packaging and signing

## Windows signing

Use `signtool` (Windows SDK):

```bash
signtool sign /fd SHA256 /a /tr http://timestamp.digicert.com /td SHA256 Tonet-x.x.x-x64.msi
signtool sign /fd SHA256 /a /tr http://timestamp.digicert.com /td SHA256 Tonet-Setup-x.x.x-x64.exe
```

## Debian package

```bash
cargo build --release -p tonet
./packaging/debian/build-deb.sh
```

## Cloudflare landing deploy

```bash
cd web/landing
npm ci
npm run deploy
```
