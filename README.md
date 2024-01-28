# remarkable-touch

Small programs to make changes to the Remarkable2 device.

I tested this on a Remarkable2 device with software version 3.9.3.1986.

Currently, 2 programs are available:
## flip
 
 This program allows to flip pages forward by pressing on the right-bottom corner
 and backward by pressing on the left-bottom corner of the screen.
 It does so by intercepting the touch events and producing a swipe events when recognizing a touch there.
 This idea was inspired by https://github.com/isaacwisdom/RemarkableLamyEraser and some code from there is also reused.
 
 ## web-interface

 This script just executes the `ip address change 10.11.99.1/32 dev usb1` command. This way, the web interface is
 available on the Remarkable. It can then be used by putting some port-forwarding like `ssh -L 8080:10.11.99.1:80` to
 access the web interface via wifi.
 This idea is inspired by https://github.com/rM-self-serve/webinterface-onboot. Somehow, my device has a `usb1` interface
 there, so it's less complicated than the proposed solution with binary patching there.

# Usage

When you have some Linux distribution on x86-64 with nix and ssh, you can run the following:

```
export REMARKABLE_HOST=<IP-OF-REMARKABLE-DEVICE>
# Copies the files to the remarkable
nix run github:fkz/remarkable-touch install all
```

This installs files to `/home/root/ourAdditionalPrograms` on the device.
The files still need to be executed manually. Currently, there is no support yet for executing them automatically.

# Uninstallation

To remove everything, it's enough to remove that directory `/home/root/ourAdditionalPrograms`