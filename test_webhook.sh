#!/bin/bash
curl -X POST http://localhost:8787/telegram/webhook \
  -H "Content-Type: application/json" \
  -d @test_webhook.json -v