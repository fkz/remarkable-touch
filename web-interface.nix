{ writeScript }:

writeScript "web-interface" ''
  #!/bin/sh

  /sbin/ip address change 10.11.99.1/32 dev usb1
''