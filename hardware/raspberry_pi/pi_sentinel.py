#!/usr/bin/env python3
# =================================================================
# ⚓ DevDock IoT Sentinel - Raspberry Pi Linux Edge Daemon
# Subscribes to Cloud MQTT Telemetry & Controls Physical GPIO LEDs
# =================================================================

import json
import time
import paho.mqtt.client as mqtt
from gpiozero import LED

# 1. GPIO Pin Definitions (BCM numbering)
# GPIO 17 (Pin 11) -> 220Ω -> Green LED Anode (+)
# GPIO 27 (Pin 13) -> 220Ω -> Yellow LED Anode (+)
# GPIO 22 (Pin 15) -> 220Ω -> Red LED Anode (+)
green_led  = LED(17)
yellow_led = LED(27)
red_led    = LED(22)

MQTT_BROKER = "broker.hivemq.com"
MQTT_PORT   = 1883
MQTT_TOPIC  = "devdock/telemetry/status"

def reset_leds():
    green_led.off()
    yellow_led.off()
    red_led.off()

def on_connect(client, userdata, flags, rc):
    if rc == 0:
        print(f"✅ Connected to Cloud MQTT Broker: {MQTT_BROKER}")
        client.subscribe(MQTT_TOPIC, qos=1)
        print(f"📡 Subscribed to telemetry topic: '{MQTT_TOPIC}'")
        green_led.on()
    else:
        print(f"❌ Connection failed with code {rc}")
        red_led.blink(on_time=0.2, off_time=0.2, n=5)

def on_message(client, userdata, msg):
    try:
        payload = json.loads(msg.payload.decode('utf-8'))
        status = payload.get("status", "clean").lower()
        dirty_count = payload.get("dirty_repos", 0)
        
        print(f"📨 [DevDock Telemetry Received]: Status = {status.upper()} (Dirty Repos: {dirty_count})")
        reset_leds()
        
        if status == "clean":
            green_led.on()
        elif status == "dirty":
            yellow_led.on()
        elif status == "error":
            red_led.on()
        elif status == "sync":
            yellow_led.blink(on_time=0.15, off_time=0.15, n=6)
            green_led.on()
            
    except Exception as e:
        print(f"⚠️ Error parsing payload: {e}")
        red_led.blink(on_time=0.1, off_time=0.1, n=3)

def main():
    print("=" * 55)
    print("⚓ DevDock Raspberry Pi Edge Sentinel Daemon")
    print("=" * 55)
    
    # Self-test on boot
    green_led.on()
    yellow_led.on()
    red_led.on()
    time.sleep(1)
    reset_leds()
    
    client = mqtt.Client(client_id="rpi-devdock-sentinel-01")
    client.on_connect = on_connect
    client.on_message = on_message
    
    client.connect(MQTT_BROKER, MQTT_PORT, keepalive=60)
    client.loop_forever()

if __name__ == "__main__":
    main()
