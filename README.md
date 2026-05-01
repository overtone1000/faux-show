# Faux Show
A web application to serve as a simple home smart display device.

## Home Assistant Modifications

### Bypass Login

### Kiosk Mode
[Kiosk mode](https://github.com/NemesisRE/kiosk-mode)
If top bar is hidden, need to access dash with `?disable_km` at the end of the URL to enable editing.
http://10.10.10.10:8123/dashboard-kiosk/0?disable_km

### Restart device without reboot
```
sudo systemctl restart faux-show-backend cage-tty1
```

### Update tabs
ssh into device and 
`sudo nano /var/lib/containers/storage/volumes/faux_show_config/_data/tabs.json`

### Trying to sleep screen
```
XDG_RUNTIME_DIR=/run/user/$(id -u) #Should be kiosk user doing this? Yes!!
WAYLAND_DISPLAY=wayland-0 #Which display? Seems this works.
wlr-randr --help #Should be sudo? No! Should be run as kiosk!
wlr-randr --output HDMI-A-1 --off #Turns it off!
wlr-randr --output HDMI-A-1 --on #Turns it on!
```
__Should be able to use this in backend to control screen with home assistant.__