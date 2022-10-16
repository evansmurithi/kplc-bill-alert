# kplc-bill-alert

Send alert notifications for KPLC post pay bills

## Usage

Sample config file:

```toml
[kplc]
basic_auth = "Basic some-basic-auth"
token_url = "https://selfservice.kplc.co.ke/api/token"
bill_url = "https://selfservice.kplc.co.ke/api/publicData/2.0.1/"
token_grant_type = "client_credentials"
token_scope = "token_public"

[pushover]
enabled = true
token = "some-token"
user_key = "some-user-key"
api_url = "https://api.pushover.net/1/messages.json"
```

To execute:

```sh
kplc-bill-alert --account-number=123456 --config /path/to/config.toml
```

## Release

```sh
git tag vx.y.z
git push origin --tags
```
