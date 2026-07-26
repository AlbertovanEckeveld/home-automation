
struct MotorStatus {                                                    // de statusen die een motor kan hebben
  bool actief;
  bool openBevestigd;
  bool dichtBevestigd;
  bool defect;
  int openPogingen;
  int sluitPogingen;
  unsigned long startTijd;
};

MotorStatus motor1;                                                     // motor1 object
MotorStatus motor2;                                                     // motor2 object

const int aantalMomenten = sizeof(schema) / sizeof(schema[0]);          // aantal momenten dat de klep op een dag opengaat

bool moetOpenZijn() {                                                   // functie die een boolean returnt of de klep open zou moeten zijn

  struct tm timeinfo;

  if (!getLocalTime(&timeinfo)) {                                       // als hij geen tijd heeft return hij false
    return false;
  };

  int nu = timeinfo.tm_hour * 60 + timeinfo.tm_min;                     // haal huidige tijd op

  for (int i = 0; i < aantalMomenten; i++) {                            // loop door het schema heen
    int start = schema[i].startHour * 60 + schema[i].startMinute;       // bepaal start tijd uit het schema
    int einde = start + VOER_DUUR_MINUTEN;                              // bepaal het eindtijd met start + de duur van het voederen

    if (nu >= start && nu < einde) {                                    // is de tijd tussen het starttijd en het eindtijd
      return true;                                                      // return dan true
    };
  };

  return false;                                                         // als het niet tussen de starttijd en eindtijd is return false
};

void voederbakLogica() {                                                // voederbak logica die gebruikt wordt in de loop functie
  
  if (moetOpenZijn()) {                                                 // controleer of de klep nu open moet zijn
    
    if (status == GESLOTEN) {                                           // als de status is GESLOTEN
      startOpenen();                                                    // start met alle motoren open laten draaien
    };
    
  } else {                                                              // als de status niet GESLOTEN is
    
    if (status == OPEN) {                                               
      startSluiten();                                                   // start met alle motoren dicht laten draaien
    };
  };
    
  verwerkOpenen();                                                      // handel de openen syclus af 
  verwerkSluiten();                                                     // handel de sluiten syclus af 
}
