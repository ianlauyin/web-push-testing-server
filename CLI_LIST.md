### Test Send

curl -X GET http://localhost:8080/subscriptions

curl -X POST http://localhost:8080/send-direct -H 'Content-Type: application/json' -d @test-send-direct.json
