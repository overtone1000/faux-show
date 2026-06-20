This contains third party software that is most easily deployed with containers.

## [Linux Voice Assistant](https://github.com/OHF-Voice/linux-voice-assistant/blob/main/docs/install_application.md)
[Example docker compose file] https://github.com/OHF-Voice/linux-voice-assistant/blob/main/docker-compose.yml
[Example environment file] https://github.com/OHF-Voice/linux-voice-assistant/blob/main/.env.example

Start testing from repo root with
```
bash ./containers/testing_start.sh
```

This connected but primarily only controls the assistant configured in home assistant. It does not seem to be useful for triggering actions on the local device.