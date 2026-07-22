
void startSluiten() {                                                     // functie de het sluiten opdracht laat starten

  Serial.println("Sluit opdracht");

  //motor1.dichtBevestigd = false;
  //motor2.dichtBevestigd = false;

  startMotor1Dicht();                                                     // functie die motor1 laat draaien
  startMotor2Dicht();                                                     // functie die motor2 laat draaien

  status = SLUITEN;                                                       // status van het programma is kleppen laten sluiten
};

void startMotor1Dicht() {                                                 // laat motor 1 dicht draaien

  if (motor1.defect) {                                                    // als motor1 defect is, laat de verdere instructie maar niet uitvoeren
    return;
  };
  
  motor1.actief = true;                                                   // zet de actief van het motor1 object op true
  motor1.startTijd = millis();                                            // noteer de start tijd wanneer de instructie aan het relay is gegeven

  aanzettenMotor1DichtDraaien();                                          // laat het relay schakelen zodat de motor gaat draaien

  Serial.println("Motor 1 sluiten");
}

void startMotor2Dicht() {                                                 // laat motor 1 dicht draaien

  if (motor2.defect) {                                                    // als motor2 defect is, laat de verdere instructie maar niet uitvoeren
    return;
  };

  motor2.actief = true;                                                   // zet de actief van het motor2 object op true
  motor2.startTijd = millis();                                            // noteer de start tijd wanneer de instructie aan het relay is gegeven

  aanzettenMotor2DichtDraaien();                                          // laat het relay schakelen zodat de motor gaat draaien

  Serial.println("Motor 2 sluiten");
};


void verwerkSluiten() {                                                   // functie die het sluiten opdracht bewaakt

  if (status != SLUITEN) {                                                // is de status van het programma niet gelijk aan SLUITEN, laat de verdere instructie maar niet uitvoeren
    return;
  };

  if (!motor1.actief && !motor2.actief) {                                 // zijn alle 2 de motoren inactief, laat de verdere instructie maar niet uitvoeren
    Serial.println("Sluiten mislukt: Beide motoren zijn inactief");
    return;
  };

  if (motor1.actief) {                                                    // als motor 1 actief is
                                                       
        // Stop automatisch na ingestelde tijd
    if (millis() - motor1.startTijd > MOTOR1_SLUITTIJD) {                 // laat de motor zolang dicht draaien zoals aan gegeven in: MOTOR1_SLUITTIJD
  
      uitzettenMotor1DichtDraaien();                                      // wanneer de sluittijd is behaalt laat de motor stoppen met draaien
  
      motor1.actief = false;                                              // geef aan dat motor 1 niet meer actief iets aan het doen is
      motor1.dichtBevestigd = true;                                       // bevestig dat de klep dicht is (dit kan niet echt kloppen door het gebrek van een sensor)
      motor1.sluitPogingen = 0;                                           // zet de aantal pogingen die hij er over heeft gedaan de motor de klep te laten zakken op 0
  
      Serial.println("Motor 1 sluiten voltooid (op basis van tijd)");
    };
  };
  

  if (motor2.actief) {                                                    // als motor 2 actief is
    
    // Stop automatisch na ingestelde tijd        
    if (millis() - motor2.startTijd > MOTOR2_SLUITTIJD) {                 // laat de motor zolang dicht draaien zoals aan gegeven in: MOTOR2_SLUITTIJD      
    
      uitzettenMotor2DichtDraaien();                                      // wanneer de sluittijd is behaalt laat de motor stoppen met draaien
    
      motor2.actief = false;                                              // geef aan dat motor 2 niet meer actief iets aan het doen is
      motor2.dichtBevestigd = true;                                       // bevestig dat de klep dicht is (dit kan niet echt kloppen door het gebrek van een sensor)
      motor2.sluitPogingen = 0;                                           // zet de aantal pogingen die hij er over heeft gedaan de motor de klep te laten zakken op 0
    
      Serial.println("Motor 2 sluiten voltooid (op basis van tijd)");
    };
  };


  if (motor1.defect) {                                                    // als motor 1 defect is gegaan tijdens het dicht draaien

    Serial.println("Sluiten mislukt: Motor 1 defect");
    status = FOUT;                                                        // zet programma in een fout status
    
  };
  
  if (motor2.defect) {                                                    // als motor 2 defect is gegaan tijdens het dicht draaien
    
    Serial.println("Sluiten mislukt: Motor 2 defect");
    status = FOUT;                                                        // zet programma in een fout status

  };

  if (motor1.dichtBevestigd &&
      motor2.dichtBevestigd) {                                            // als beide motoren een dichtBevestigd true hebben

    Serial.println("Beide kleppen dicht");
    status = GESLOTEN;                                                    // zet het programma in de GESLOTEN status
      
    allRelaysOff();                                                       // zet alle relay's op LOW, zodat er geen stroom loopt naar de motoren
  };
};
