# ECSR

## Example
```
$ ./target/release/ecsr
Read profile from /Users/egapool/.aws/credentials
✔ Select profile from "/Users/egapool/.aws/credentials" · default
✔ Select cluster · your-cluster-name
✔ Select service · app
✔ Select task · 14eb7546ebcx45d48656537cba8dd7ea
✔ Select container · web-container
✔ Command · bash
aws --profile default ecs execute-command --cluster your-cluster-name --container web-container --interactive --command bash --task 14eb7546ebcx45d48656537cba8dd7ea
```

## Install

### OSX
```
$ brew tap egapool/ecsr
$ brew install ecsr
```
