

void allRelaysOff() {                                                           // functie om alle relays uit te schakelen
  for (int i = 0; i < 4; i++) {                                                 // loop door alle 4 de relays
    digitalWrite(relays[i], LOW);                                               // zet ze allemaal op LOW, zodat er geen stroom door de relay wordt gestuurd naar de motoren
  };
  
  Serial.println("Alle Relays uitgeschakeld");
};

void setupMotors() {                                                            // functie die alle GPIO's van alle motoren als OUTPUT definieerd
  for (int i = 0; i < 4; i++) {
    pinMode(relays[i], OUTPUT);
  };

  allRelaysOff();                                                               // zet alle relays ook direct uit, zodat ze geen stroom geven aan de motoren

  Serial.println("Relays ingesteld.");
};

void uitzettenMotor1DichtDraaien() {                                            
  digitalWrite(MOTOR1_1, LOW);                                                  // zet MOTOR 1 = relay 1 | Dicht laten draaien uit
};

void uitzettenMotor1OpenDraaien() {
  digitalWrite(MOTOR1_2, LOW);                                                  // zet MOTOR 1 = relay 2 | Open laten draaien uit
};

void uitzettenMotor2DichtDraaien() {                                            
  digitalWrite(MOTOR2_1, LOW);                                                  // zet MOTOR 2 = relay 1 | Dicht laten draaien uit
};

void UitzettenMotor2OpenDraaien() {                                             
  digitalWrite(MOTOR2_2, LOW);                                                  // zet MOTOR 2 = relay 2 | Open laten draaien uit
};

void aanzettenMotor1DichtDraaien() {                                            
  uitzettenMotor1OpenDraaien();                                                 // zorg dat de relay van motor 1 voor het open draaien uit is
  digitalWrite(MOTOR1_1, HIGH);                                                 // laat MOTOR 1 = relay 1 | Dicht draaien 
};

void aanzettenMotor1OpenDraaien() {
  uitzettenMotor1DichtDraaien();                                                // zorg dat de relay van motor 1 voor het dicht draaien uit is
  digitalWrite(MOTOR1_2, HIGH);                                                 // laat MOTOR 1 = relay 2 | Open draaien 
};

void aanzettenMotor2DichtDraaien() {
  UitzettenMotor2OpenDraaien();                                                 // zorg dat de relay van motor 2 voor het open draaien uit is
  digitalWrite(MOTOR2_1, HIGH);                                                 // laat MOTOR 2 = relay 1 | Dicht draaien 
};

void aanzettenMotor2OpenDraaien() {
  uitzettenMotor2DichtDraaien();                                                // zorg dat de relay van motor 2 voor het dicht draaien uit is
  digitalWrite(MOTOR2_2, HIGH);                                                 // laat MOTOR 2 = relay 2 | Open draaien 
};
