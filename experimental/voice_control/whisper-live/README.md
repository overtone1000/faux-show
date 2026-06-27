https://github.com/hwdsl2/docker-whisper-live

Run from repo root with
```
bash ./experimental/voice_control/whisper-live/testing_start.sh
```

Stop with
```
bash ./experimental/voice_control/whisper-live/testing_stop.sh
```

Example of output:
```
Websocket received: Text(Utf8Bytes(b"{\"uid\": \"test_client\", \"segments\": [{\"start\": \"4.096\", \"end\": \"14.316\", \"text\": \" Testing transcription.\", \"completed\": true}, {\"start\": \"14.316\", \"end\": \"15.316\", \"text\": \" Finished!\", \"completed\": true}, {\"start\": \"28.288\", \"end\": \"30.628\", \"text\": \" What more is there to say?\", \"completed\": true}, {\"start\": \"40.278\", \"end\": \"43.258\", \"text\": \" While this works well that consumes a lot of CPU\", \"completed\": false}]}"))
```