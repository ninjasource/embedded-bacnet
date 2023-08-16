#!/bin/bash

# extract ip from local arp table
ip=$(arp | grep 00:10:70:18:6A:7F | awk ' { print $1 } ')

# found an ip tied to the mac address?
if [ ! -z $ip ]; then

    # if found, do you want to ping it?
    ping $ip
else
    echo "Not found into local arp table. Trying another way..."

    # wanna try your nmap strategy?
    # sudo nmap -sP 192.168.15.1/24 | grep  20:64:32:3F:B1:A9
fi;
