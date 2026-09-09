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

With word_timestamp set to true:
Websocket received: Text(Utf8Bytes(b"
{
    \"uid\": \"test_client\", 
    \"segments\": 
        [
            {
                \"start\": \"3.072\", 
                \"end\": \"3.912\", 
                \"text\": \" Hello there\", 
                \"completed\": true
            }, 
            {
                \"start\": \"15.872\", 
                \"end\": \"17.392\", 
                \"text\": \" My name is Tyler Moore.\", 
                \"completed\": false, 
                \"words\": 
                    [
                        {
                            \"word\": \" My\", 
                            \"start\": \"15.872\", 
                            \"end\": \"16.412\", 
                            \"probability\": 0.5378
                        }, 
                        {\"word\": \" name\", \"start\": \"16.412\", \"end\": \"16.612\", \"probability\": 0.9908}, {\"word\": \" is\", \"start\": \"16.612\", \"end\": \"16.792\", \"probability\": 0.9886}, {\"word\": \" Tyler\", \"start\": \"16.792\", \"end\": \"17.072\", \"probability\": 0.9964}, {\"word\": \" Moore.\", \"start\": \"17.072\", \"end\": \"17.392\", \"probability\": 0.1891}]}]}"))