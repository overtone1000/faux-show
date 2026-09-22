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
    - [x] Just get audio stream once and reuse data
        - [x] Need a unified data function for audio stream
            This should probably take the form primarily of the circular buffer of the wakeword listener with access to the whisper stream. When wakeword detection occurs, send some amount of the circular buffer (good way to determine where the end of the wakeword is from livekit-wakeword?) to the whisper stream and then continue to do so for some amount of time (10s?) or until a command is definitely identified from the returning transcription.
        - [x] Implemented, need to test!
    - [x] Better handling of whisper client
        - Need to connect for each instance of a stream to/from whisper server instead of keeping open indefinitely
    - [x] Convert sample functions to structs with initialization
    - [x] Test UI
    - [x] Internal state management is brittle especially in "audio_stream". Probably need a single listener for all state changes that then broadcasts full state updates to the rest of the internal services.
    - [x] Need to recognize some commands and make things happen!
        - [x] Would be best to have it be part of the config as json
            - Should have configurable types
                - [x] Quick contains: just a word or phrase that has to be in the transcriptiuon
            - Should have configurable actions
                - [x] Open a specific page
    - [x] Close whisper socket faster. Search for "println!("Need to initiate websocket closure here!")" to find the right place
        - Not working.
            - After enabled, voice control state is set repeatedly.
            - Socket still isn't closing down...