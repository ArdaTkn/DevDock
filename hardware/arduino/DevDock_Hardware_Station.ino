// =================================================================
// ⚓ DevDock IoT Sentinel - Arduino Uno/Nano Hardware Station
// Direct USB Serial Telemetry Controller for DevEx Status
// =================================================================

const int GREEN_LED  = 2;  // Digital Pin 2 -> 220Ω -> Green LED Anode (+)
const int YELLOW_LED = 3;  // Digital Pin 3 -> 220Ω -> Yellow LED Anode (+)
const int RED_LED    = 4;  // Digital Pin 4 -> 220Ω -> Red LED Anode (+)

void resetLeds() {
  digitalWrite(GREEN_LED, LOW);
  digitalWrite(YELLOW_LED, LOW);
  digitalWrite(RED_LED, LOW);
}

void setup() {
  pinMode(GREEN_LED, OUTPUT);
  pinMode(YELLOW_LED, OUTPUT);
  pinMode(RED_LED, OUTPUT);
  
  // Start USB Serial communication at 9600 Baud
  Serial.begin(9600);
  
  // Self-test sequence on boot: flash all LEDs for 1 second
  digitalWrite(GREEN_LED, HIGH);
  digitalWrite(YELLOW_LED, HIGH);
  digitalWrite(RED_LED, HIGH);
  delay(1000);
  resetLeds();
  
  // Default to Green (Clean) on startup
  digitalWrite(GREEN_LED, HIGH);
  Serial.println("⚓ DevDock Hardware Station Ready. Listening on Serial...");
}

void loop() {
  // Listen for incoming commands from DevDock via USB Serial
  if (Serial.available() > 0) {
    char cmd = Serial.read();
    
    // Command 'C' = Clean state (All repositories clean)
    if (cmd == 'C') {
      resetLeds();
      digitalWrite(GREEN_LED, HIGH);
      Serial.println("[ACK] Hardware State: CLEAN (Green ON)");
    }
    // Command 'D' = Dirty state (Uncommitted Git changes)
    else if (cmd == 'D') {
      resetLeds();
      digitalWrite(YELLOW_LED, HIGH);
      Serial.println("[ACK] Hardware State: DIRTY (Yellow ON)");
    }
    // Command 'E' = Error state (Missing .env, port conflict)
    else if (cmd == 'E') {
      resetLeds();
      digitalWrite(RED_LED, HIGH);
      Serial.println("[ACK] Hardware State: ERROR (Red ON)");
    }
    // Command 'S' = Syncing state (Bulk Git Pull / Cache clean)
    else if (cmd == 'S') {
      resetLeds();
      for (int i = 0; i < 6; i++) {
        digitalWrite(YELLOW_LED, HIGH);
        digitalWrite(GREEN_LED, LOW);
        delay(150);
        digitalWrite(YELLOW_LED, LOW);
        digitalWrite(GREEN_LED, HIGH);
        delay(150);
      }
      digitalWrite(GREEN_LED, HIGH);
      Serial.println("[ACK] Hardware State: SYNC COMPLETE");
    }
  }
}
