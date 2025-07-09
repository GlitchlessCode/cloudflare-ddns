# cloudflare-ddns

A Dynamic DNS systemd service for Cloudflare written in Rust.

# Installation

There is no distributable as of right now. I'll maybe get around to it at some point, but until then, refer to [Building](#Building) to create an executable. 
Once you have an executable place it in a location (eg. `/usr/local/bin/`), take note of the executable's location, and follow the next steps:

1. Modify `cloudflare-ddns.service` to change `ExecStart=` to the location of the `cloudflare-ddns` binary (eg. `ExecStart=/usr/local/bin/clouflare-ddns`)
2. Move both `cloudflare-ddns.service` and `cloudflare-ddns.timer` into `/etc/systemd/system/`
3. Run `systemctl daemon-reload` to register the new services
4. Run both `systemctl enable cloudflare-ddns.timer` and `systemctl start cloudflare-ddns.timer` to start the timer service
5. When it runs for the first time, a new config file should be generated at `/etc/cloudflare-ddns/config.toml`, there is an example (`config.example.toml`) provided in this repo. You can also refer to [Configuration](#Configuration) for more info on what all the config options are.

# Building

1. Clone this repo (`git clone https://github.com/GlitchlessCode/cloudflare-ddns.git`)
2. Enter the directory (`cd cloudflare-ddns`)
3. Build the binary (`cargo build --release`)
4. The binary will be located at (`./target/release/cloudflare-ddns`)

# Configuration

```
active - bool (default: false) - WHETHER TO RUN THE DDNS 

[ip-find]
finders - Vec<String> (default: []) - LIST OF URLS TO TRY FETCHING THE PUBLIC IP FROM
retries - bool (optional) - NUMBER OF RETRY ATTEMPTS FOR EACH FINDER URL
timeout - bool (optional) - TIMEOUT IN SECONDS FOR EACH FINDER ATTEMPT

[cloudflare]
api-key - String (default: "") - YOUR API KEY FOR CLOUDFLARE. MUST HAVE EDIT DNS PERMISSIONS
zone-identifier - String (default: "") - THE ID OF THE ZONE TO EDIT
dns-record-name - String (default: "") - THE NAME OF THE RECORD TO EDIT

[cache]
ignore - bool (optional) - WHETHER TO IGNORE THE CACHE AND FORCE A CLOUDFLARE UPDATE EVEN IF ONE ISN'T NECESSARY
persist - bool (optional) - WHETHER TO WRITE TO THE CACHE AND SAVE THE LAST SENT IP
```

# License

```
MIT License

Copyright (c) 2025 GlitchlessCode

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
