# Shmashmexa
A web application to serve as a simple home smart display device.

## Home Assistant Modifications

### Bypass Login

### Kiosk Mode
[Kiosk mode](https://github.com/NemesisRE/kiosk-mode)
If top bar is hidden, need to access dash with `?disable_km` at the end of the URL to enable editing.
http://10.10.10.10:8123/dashboard-kiosk/0?disable_km

### Update tabs
ssh into device and 
`sudo nano /var/lib/containers/storage/volumes/shmashmexa_config/_data/tabs.json`