# 📡 DevDock IoT Sentinel — Hardware Station Guide

This directory contains the firmware, edge daemons, and circuit specifications to connect **DevDock** with physical hardware stations (**Arduino Uno / Nano** or **Raspberry Pi**).

---

## 🚦 Status Indicators & LED Behavior

| LED Color | Trigger Condition | Meaning |
| :--- | :--- | :--- |
| 🟢 **Green (Solid)** | All repositories clean & in sync | Dev environment is healthy and ready |
| 🟡 **Yellow (Solid)** | Uncommitted Git changes detected | Code needs staging / committing / pushing |
| 🔴 **Red (Solid)** | Missing `.env` secrets or port error | Security / dependency issue detected |
| ✨ **Blinking** | Bulk Git Pull or Cache Janitor active | Async operation running in background |

---

## ⚡ Option 1: Arduino Uno / Nano (USB Serial Mode)

### 🔌 Pinout & Circuit Schematic
* **Digital Pin 2:** ➔ `220Ω Resistor` ➔ **Green LED** Anode (+)
* **Digital Pin 3:** ➔ `220Ω Resistor` ➔ **Yellow LED** Anode (+)
* **Digital Pin 4:** ➔ `220Ω Resistor` ➔ **Red LED** Anode (+)
* **GND Pin:** ➔ Common Cathode (-) rail of all 3 LEDs

### 🚀 Quick Start
1. Connect Arduino to computer via USB cable.
2. Open `hardware/arduino/DevDock_Hardware_Station.ino` in the Arduino IDE.
3. Click **Upload (➔)**.
4. In DevDock, go to **Settings ➔ Hardware & IoT Station**, select your serial port (e.g. `/dev/tty.usbmodem...` or `COM3`), and test with the buttons!

---

## 🍓 Option 2: Raspberry Pi (Cloud MQTT Mode)

### 🔌 GPIO Pinout (BCM)
* **GPIO 17 (Pin 11):** ➔ `220Ω Resistor` ➔ **Green LED** Anode (+)
* **GPIO 27 (Pin 13):** ➔ `220Ω Resistor` ➔ **Yellow LED** Anode (+)
* **GPIO 22 (Pin 15):** ➔ `220Ω Resistor` ➔ **Red LED** Anode (+)
* **GND (Pin 6 or 9):** ➔ Common Cathode (-) rail of all 3 LEDs

### 🚀 Quick Start
1. Install dependencies on Raspberry Pi:
   ```bash
   pip install paho-mqtt gpiozero
   ```
2. Run the edge daemon:
   ```bash
   python3 hardware/raspberry_pi/pi_sentinel.py
   ```

---

## 🌐 Wokwi Online Simulation
You can test the entire circuit online in your browser without any physical hardware on Wokwi:
👉 **[Wokwi Arduino Uno Simulator](https://wokwi.com/projects/new/arduino-uno)**
