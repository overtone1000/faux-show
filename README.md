# Faux Show
A web application to serve as a simple home smart display device.

## Home Assistant Modifications

### Bypass Login
Use auth to allow the kiosk to bypass login screen

### Kiosk Mode Dashboard
[Kiosk mode](https://github.com/NemesisRE/kiosk-mode)
If top bar is hidden, need to access dash with `?disable_km` at the end of the URL to enable editing.
http://10.10.10.10:8123/dashboard-kiosk/0?disable_km

## Restart faux-show on device without reboot
```
sudo systemctl restart faux-show-backend cage-tty1
```

## Update tabs
ssh into device and 
`sudo nano /var/www/internal/faux_show_config/tabs.json`

## To Do
- [ ] Consider watchdog on existing internal websocket to reset cage-tty1 service if frontend stops responding.
- [ ] Voice control
    - [x] Refactor common constants for wakeword and streaming
    - [ ] Just get audio stream once and reuse data
        - [ ] Need a unified data function for audio stream
            This should probably take the form primarily of the circular buffer of the wakeword listener with access to the whisper stream. When wakeword detection occurs, send some amount of the circular buffer (good way to determine where the end of the wakeword is from livekit-wakeword?) to the whisper stream and then continue to do so for some amount of time (10s?) or until a command is definitely identified from the returning transcription.
    - [ ] Convert sample functions to structs with initialization
    - [ ] Test UI