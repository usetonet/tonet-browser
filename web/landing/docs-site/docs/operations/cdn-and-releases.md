# CDN and releases

Tonet can deliver installers and update metadata from a Cloudflare-backed CDN instead of GitHub release endpoints.

## Landing environment variables

```dotenv
TONET_CDN_BASE_URL=https://downloads.example.com
TONET_CDN_VERSION_PATH=/version.json
TONET_SITE_DOWNLOADS_PAGE=https://downloads.example.com/
```

## App update environment variables

```dotenv
TONET_UPDATE_MANIFEST_URL=https://downloads.example.com/version.json
TONET_DOWNLOADS_PAGE_URL=https://downloads.example.com/
```

## Manifest shape

`version.json` should expose at least:

```json
{
  "version": "0.2.1",
  "download": {
    "windowsSetup": "https://downloads.example.com/Tonet-Setup-0.2.1-x64.exe",
    "linuxSetup": "https://downloads.example.com/tonet_0.2.1_amd64.deb"
  }
}
```
