/*
 * AlbertoVE Automatische Voederbak V1.0
 * Date: 20-07-2026
 * 
 * PCB ontwikkeld door Chris Jansen
 * Geprogrammeerd door Alberto van Eckeveld
 * 
 * Note: 
 * 
 * Het zijn automatische hooi voederbak voor paarden, 
 * in totaal 2 hooibakken met een klep en daaraan een motor, 
 * die de klep omhoog een naar beneden kan doen.
 * 
 * De motoren die gebruikt zijn, komen van een automatisch rolgordijn, met daaraan een touw door een catrool bevestigd aan de klep.
 * 
 * Iedere klep heeft ook een open sensor, die meet of de klep al op zijn bovenste punt is beland.
 * De dicht sensor ontbreekt in het eindresultaat, maar is wel meegenomen in het PCB design.
 * 
 */

const bool DEBUG = false;

const int MOTOR1_1 = 3;                                       // GPIO 3 = MOTOR 1 = relay 1 | Dicht laten draaien
const int MOTOR1_2 = 2;                                       // GPIO 2 = MOTOR 1 = relay 2 | Open laten draaien
const int MOTOR2_1 = 1;                                       // GPIO 1 = MOTOR 2 = relay 1 | Dicht laten draaien
const int MOTOR2_2 = 0;                                       // GPIO 0 = MOTOR 2 = relay 2 | Open laten draaien

const int relays[] = {
  MOTOR2_2,
  MOTOR2_1,
  MOTOR1_2,
  MOTOR1_1
};

const int MAX_OPEN_POGINGEN  = 3;                             // aantal pogingen na openen timeout per motor
//const int MAX_SLUIT_POGINGEN = 3;                           // aantal pogingen na dicht timeout per motor (wordt niet gebruikt)

const unsigned long MOTOR1_SLUITTIJD = 19000;                 // 19 seconden | Motor 1 doet 19 seconden over klep dichten
const unsigned long MOTOR2_SLUITTIJD = 27000;                 // 27 seconden | Motor 1 doet 27 seconden over klep dichten
const unsigned long MOTOR1_OPEN_TIMEOUT = 20000;              // 20 seconden | als draai timeout
const unsigned long MOTOR2_OPEN_TIMEOUT = 30000;              // 30 seconden | als draai timeout

const int SENSOR_MOTOR1_DICHT = 4;                            // GPIO 4 = SENSOR voor motor 1 of klep dicht is (wordt niet gebruikt)
const int SENSOR_MOTOR1_OPEN = 5;                             // GPIO 5 = SENSOR voor motor 1 of klep open is (
const int SENSOR_MOTOR2_DICHT = 6;                            // GPIO 6 = SENSOR voor motor 2 of klep dicht is (wordt niet gebruikt)
const int SENSOR_MOTOR2_OPEN = 7;                             // GPIO 7 = SENSOR voor motor 2 of klep open is

const int sensors[] = {
  SENSOR_MOTOR1_DICHT,
  SENSOR_MOTOR1_OPEN,
  SENSOR_MOTOR2_DICHT,
  SENSOR_MOTOR2_OPEN
};

// Wifi instellingen
unsigned long laatsteTimeSync = 0;
unsigned long laatsteWifiCheck = 0;

const char* ssid = "Koos Draadloos IoT";
const char* password = "IoTNetwerk";

const unsigned long WIFI_CHECK_INTERVAL = 10000;                     // elke 10 seconden wordt de wifi verbinding gecheckt
const unsigned long TIME_SYNC_INTERVAL = 6UL * 60UL * 60UL * 1000UL; // om de 6 uur wordt de tijd gesyncroniseerd

enum KlepStatus {                                                    // alle statusen die de motoren kunnen hebben
  GESLOTEN,
  OPENEN,
  OPEN,
  SLUITEN,
  FOUT
};

KlepStatus status = GESLOTEN;                                       // standaard is de status van de klep dicht

struct VoerMoment {                                                 // structuur van het schema hieronder
  int startHour;
  int startMinute;
};


VoerMoment schema[] = {                                             // Schema wanneer de kleppen open moeten zijn.
  {7, 0},                                                           // Open: 06:00 Dicht: 06:20
  {10, 0},                                                          // Open: 09:00 Dicht: 09:20
  {13, 0},                                                          // Open: 12:00 Dicht: 12:20
  {16, 0},                                                          // Open: 15:00 Dicht: 15:20
  {19, 0},                                                          // Open: 18:00 Dicht: 18:20
  {22, 0},                                                          // Open: 21:00 Dicht: 21:20
  {1, 0},                                                           // Open: 00:00 Dicht: 00:20
  {4, 0}                                                            // Open: 03:00 Dicht: 03:20
};

const int VOER_DUUR_MINUTEN = 15;                                   // hoelang de klep open is

void setup() {                                                      // stukje opstart code
  Serial.begin(115200);
  delay(100);
  Serial.println("AlbertoVE Automatische voederbak gestart");

  setupMotors();                                                    // zet alle motor GPIO's op output
  setupSensors();                                                   // zet alle sensor GPIO's op input

  if (connectWifi()) {                                              // zet WiFi verbinding op
    setupTime();                                                    // syncroniseer de TRC clock met een NTP server
    printCurrentTime();                                             // print de huidige tijd in serial
  }

  allRelaysOff();                                                   // zet alle relay outputs op LOW

  if (moetOpenZijn()) {                                             // check of de huidige tijd tussen het schema van openen is
    
    if (status == GESLOTEN) {                                       // als de status is GESLOTEN
      startOpenen();                                                // start met alle motoren open laten draaien
    }
    
  } else {                                                          // als de status niet GESLOTEN is
      startSluiten();                                               // start met alle motoren dicht laten draaien
  }
    
  Serial.println("Start-up voltooid.");
};

void loop() {                                                       // loop van de programma

  if (checkWifi()) {                                                // controleer of de WiFi verbinding er nog is
    updateTime();                                                   // syncroniseer de TRC clock met een NTP server
  };

  if (!DEBUG) {                                                     // als de DEBUG optie uit staat
  
    voederbakLogica();                                              // voer dan de voederbak logica uit
    
    delay(100);                                                     // halt de CPU voor 100 miliseconden.
    
  } else {                                              // als de DEBUG functie wel aanstaat
    
    printCurrentTime();                                 // print de huidige tijd
    int klepMotor1 = digitalRead(SENSOR_MOTOR1_OPEN);   // haal de sensor waarde op open_sensor van motor 1
    int klepMotor2 = digitalRead(SENSOR_MOTOR2_OPEN);   // haal de sensor waarde op open_sensor van motor 2
    Serial.print("s1-2: ");
    Serial.println(klepMotor1);                         // print de waarde van open_sensor van motor 1 in serial
    Serial.print("s2-2: ");                           
    Serial.println(klepMotor2);                         // print de waarde van open_sensor van motor 2 in serial
    if (klepMotor1 == LOW && klepMotor2 == LOW) {Serial.println("Beide kleppen staan open");}

    voederbakLogica();                                  // voer dan de voederbak logica uit
    
    delay(2000);                                        // halt de CPU voor 2 seconden.
  }

}
