# `rpi-vbus-logger`

Eine Anwendung zum Loggen von RESOL VBus-Daten in eine MySQL/MariaDB-Datenbank
mit Hilfe eines Raspberry Pi oder ähnlicher Linux-basierter Single-Board-Computer.


## Überblick

Die Anwendung `rpi-vbus-logger` ist eine in Rust geschriebene
Kommandozeilenanwendung, die RESOL VBus-Daten über eine serielle Schnittstelle
empfängt und diese in regelmäßigen Abständen in einer MySQL/MariaDB-Datenbank
ablegt.

Vereinfacht dargestellt sieht die Tabellenstruktur wie folgt aus:

```
+----+------------------+---------+---------+-----+---------+
| id | timestamp        | value_1 | value_2 | ... | value_n |
+----+------------------+---------+---------+-----+---------+
| 38 | 19.03.2017 08:00 |    27.3 |       0 | ... |     440 |
| 39 | 19.03.2017 08:10 |    30.5 |       0 | ... |     450 |
+----+------------------+---------+---------+-----+---------+
```

Die gewählte Struktur ist auf einfache Weiterverarbeitung der Daten
ausgelegt. Pro aufgezeichnetem Datensatz wird eine Zeile in einer Tabelle `data`
abgelegt, in dem alle in der Konfiguration angegebenen VBus-Werte abgelegt sind.
Die Spaltennamen können in der Konfigurationsdatei der Anwendung vorgegeben
werden.


## Installation

Als Voraussetzung müssen die Werkzeuge für die Programmiersprache
["Rust"](https://www.rust-lang.org) installiert werden. Dazu wird zuerst der
Befehl `rustup` installiert. Im Laufe seiner Installation werden auch die
aktuelle stabilen Rust-Werkzeuge (Compiler usw.) mitinstalliert
([Quelle](https://www.rust-lang.org/en-US/install.html)):

```
pi@raspberrypi:~ $ curl https://sh.rustup.rs -sSf | sh
```

Nach Abschluss der Installation sollten die Befehle `rustc` und `cargo` zur
Verfügung stehen:

```
pi@raspberrypi:~ $ rustc --version
rustc 1.16.0 (30cf806ef 2017-03-10)
pi@raspberrypi:~ $ cargo --version
cargo-0.17.0-nightly (f9e5481 2017-03-03)
```

Danach kann das Archiv mit dem Quellcode des `rpi-vbus-logger` entpackt werden:

```
pi@raspberrypi:~ $ tar xzf rpi-vbus-logger.tar.gz
```

Dadurch wird im aktuellen Verzeichnis ein neues Unterverzeichnis
`rpi-vbus-logger` erstellt. Der darin enthaltenen Quellcode kann jetzt mit
`cargo build` durchkompiliert werden:

```
pi@raspberrypi:~ $ cd rpi-vbus-logger
pi@raspberrypi:~/rpi-vbus-logger $ cargo build
```

Wenn das Kompilieren fehlerfrei geklappt hat, liegt danach unter
`target/debug/rpi-vbus-logger` das entsprechende Executable.


## Konfiguration

Bevor die Anwendung aber gestartet werden kann, muss sie noch konfiguriert
werden. Dafür wird eine Datei `rpi-vbus-logger.toml` im aktuellen Verzeichnis
erwartet. Das Quellcode-Archiv enthält bereits eine Beispiel-Datei, die
nur noch umbenannt werden muss:

```
pi@raspberrypi:~/rpi-vbus-logger $ cp rpi-vbus-logger.toml.example rpi-vbus-logger.toml
```

Die Datei hat dann folgenden Inhalt:

```toml
[database]
hostname = "127.0.0.1"
port = 3306
username = "dbusername"
password = "dbpassword"
database = "vbus_data"

[serial]
path = "/dev/tty.SLAB_USBtoUART"

[logger]
interval = 60


[[fields]]
column = "temp_sp_1"
packet_id = "00_0010_7E11_10_0100"
field_id = "002_2_0"

[[fields]]
column = "temp_sp_2"
packet_id = "00_0010_7E11_10_0100"
field_id = "004_2_0"
```

Die Sektion `[serial]` gibt in der Option `path` den Gerätenamen der seriellen
Schnittstelle an, über die VBus-Daten empfangen werden sollen.

Die Sektion `[logger]` enthält in der Option `interval` den gewünschten
zeitlichen Abstand zwischen zwei Datensätzen in Sekunden.

Die Sektion `[database]` enthält die Konfigurationseinstellungen für den
Zugang zur MySQL-/MariaDB-Datenbank. Die genannte Datenbank muss für den
genannten Benutzer lesbar und schreibbar sein und eine Tabelle mit dem Namen
`data` enthalten, die mindestens eine autoinkrementierende `id`-Spalte und eine
DATETIME-Spalte mit dem Namen `timestamp` enthalten muss. Alle weiteren Spalten
sind abhängig vom Einsatzfall (gewünschte Daten / verwendete Geräte) und müssen
vom Datentyp DOUBLE sein.

Zu jeder weiteren Spalte in der Tabelle muss auch ein `[[fields]]`-Eintrag in
der Konfigurationsdatei vorliegen. Darin wird festgelegt, welcher VBus-Wert
in welcher zusätzlichen Datenbank-Spalte abgelegt werden soll.

Für eine Datenaufzeichnung der zwei in der Beispiel-Konfigurationsdatei
angegebenen VBus-Werte `temp_sp_1` und `temp_sp_2` muss eine Tabelle mit
folgender Struktur angelegt werden:

```sql
CREATE TABLE `data` (
  `id` int(11) unsigned NOT NULL AUTO_INCREMENT,
  `timestamp` datetime DEFAULT NULL,
  `temp_sp_1` double DEFAULT NULL,
  `temp_sp_2` double DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
```

Wenn die `packet_id` und `field_id`-Werte für die `[[fields]]`-Liste nicht
bekannt sind, kann die Applikation auch ohne `[[fields]]` gestartet und auf
das Ende der Stabilisierungsphase gewartet werden. Zu dem Zeitpunkt wird eine
Liste aller gefundener `packet_id`/`field_id`-Kombinationen ausgegeben, die
dann in die Konfigurationsdatei übernommen werden können.


## Ausführung

Wenn die Konfigurationsdatei angepasst und die Datenbankstruktur erstellt
wurde, kann die Anwendung entweder über `target/debug/rpi-vbus-logger` oder
über `cargo run` gestartet werden. Die Ausgabe sieht dann bei einem
angeschlossenen RESOL DeltaSol MX wie folgt aus:

```
pi@raspberrypi:~/rpi-vbus-logger $ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.0 secs
     Running `target/debug/rpi-vbus-logger`
Connecting to database...
Finding last record...
    last_timestamp = 2017-03-19T08:43:00Z
Process field configuration...
Connecting to VBus...
Data set changed, waiting for it to stabilize again...
Data set changed, waiting for it to stabilize again...
Data set changed, waiting for it to stabilize again...
Data set stabilizing at 0%
Data set stabilizing at 11%
Data set changed, waiting for it to stabilize again...
Data set stabilizing at 0%
Data set stabilizing at 8%
Data set stabilizing at 16%
Data set stabilizing at 24%
Data set changed, waiting for it to stabilize again...
Data set stabilizing at 0%
Data set stabilizing at 6%
Data set changed, waiting for it to stabilize again...
Data set stabilizing at 0%
Data set stabilizing at 5%
Data set changed, waiting for it to stabilize again...
Data set stabilizing at 0%
Data set stabilizing at 4%
Data set stabilizing at 9%
Data set stabilizing at 14%
Data set stabilizing at 18%
Data set stabilizing at 23%
Data set stabilizing at 28%
Data set stabilizing at 33%
Data set stabilizing at 37%
Data set stabilizing at 42%
Data set stabilizing at 47%
Data set stabilizing at 51%
Data set stabilizing at 56%
Data set stabilizing at 61%
Data set stabilizing at 66%
Data set stabilizing at 70%
Data set stabilizing at 75%
Data set stabilizing at 80%
Data set stabilizing at 84%
Data set stabilizing at 89%
Data set stabilizing at 94%
Data set stabilized
- 00_0010_7E11_10_0100: DeltaSol MX [Regler]
    - 000_2_0: Temperature sensor 1
    - 002_2_0: Temperature sensor 2
    - 004_2_0: Temperature sensor 3
    - 006_2_0: Temperature sensor 4
    - 008_2_0: Temperature sensor 5
    - 010_2_0: Temperature sensor 6
    - 012_2_0: Temperature sensor 7
    - 014_2_0: Temperature sensor 8
    - 016_2_0: Temperature sensor 9
    - 018_2_0: Temperature sensor 10
    - 020_2_0: Temperature sensor 11
    - 022_2_0: Temperature sensor 12
    - 024_2_0: Temperature sensor 13
    - 026_2_0: Temperature sensor 14
    - 028_2_0: Temperature sensor 15
    - 030_2_0: Irradiation sensor 16
    - 032_2_0: Temperature sensor 17
    - 034_2_0: Temperature sensor 18
    - 036_2_0: Temperature sensor 19
    - 038_2_0: Temperature sensor 20
    - 040_4_0: Flow rate sensor 13
    - 044_4_0: Flow rate sensor 14
    - 048_4_0: Flow rate sensor 15
    - 052_4_0: Flow rate sensor 17
    - 056_4_0: Flow rate sensor 18
    - 060_4_0: Flow rate sensor 19
    - 064_4_0: Flow rate sensor 20
    - 104_4_0: Flow rate sensor 21
    - 068_2_0: Pressure sensor 17
    - 070_2_0: Pressure sensor 18
    - 072_2_0: Pressure sensor 19
    - 074_2_0: Pressure sensor 20
    - 076_1_0: Pump speed relay 1
    - 077_1_0: Pump speed relay 2
    - 078_1_0: Pump speed relay 3
    - 079_1_0: Pump speed relay 4
    - 080_1_0: Pump speed relay 5
    - 081_1_0: Pump speed relay 6
    - 082_1_0: Pump speed relay 7
    - 083_1_0: Pump speed relay 8
    - 084_1_0: Pump speed relay 9
    - 085_1_0: Pump speed relay 10
    - 086_1_0: Pump speed relay 11
    - 087_1_0: Pump speed relay 12
    - 088_1_0: Pump speed relay 13
    - 089_1_0: Pump speed relay 14
    - 100_1_0: Output A
    - 101_1_0: Output B
    - 102_1_0: Output C
    - 103_1_0: Output D
    - 092_4_0: System date
    - 096_4_0: Error mask
- 00_6654_7E11_10_0200: DeltaSol MX [Regler] => EM #4
    - 000_1_0: Pump speed relay 1.1
    - 001_3_0: Timer 1.1
    - 004_1_0: Pump speed relay 1.2
    - 005_3_0: Timer 1.2
    - 008_1_0: Pump speed relay 2.1
    - 009_3_0: Timer 2.1
    - 012_1_0: Pump speed relay 2.2
    - 013_3_0: Timer 2.2
    - 016_1_0: Pump speed relay 3.1
    - 017_3_0: Timer 3.1
    - 020_1_0: Pump speed relay 3.2
    - 021_3_0: Timer 3.2
    - 024_1_0: Pump speed relay 4.1
    - 025_3_0: Timer 4.1
    - 028_1_0: Pump speed relay 4.2
    - 029_3_0: Timer 4.2
    - 032_1_0: Pump speed relay 5.1
    - 033_3_0: Timer 5.1
    - 036_1_0: Pump speed relay 5.2
    - 037_3_0: Timer 5.2
    - 040_1_0: SensorOutputType1
    - 041_1_0: SensorOutputType2
    - 042_1_0: SensorOutputType3
    - 043_1_0: SensorOutputType4
    - 044_1_0: SensorOutputType5
    - 045_1_0: SensorOutputType6
- 00_6655_7E11_10_0200: DeltaSol MX [Regler] => EM #5
    - 000_1_0: Pump speed relay 1.1
    - 001_3_0: Timer 1.1
    - 004_1_0: Pump speed relay 1.2
    - 005_3_0: Timer 1.2
    - 008_1_0: Pump speed relay 2.1
    - 009_3_0: Timer 2.1
    - 012_1_0: Pump speed relay 2.2
    - 013_3_0: Timer 2.2
    - 016_1_0: Pump speed relay 3.1
    - 017_3_0: Timer 3.1
    - 020_1_0: Pump speed relay 3.2
    - 021_3_0: Timer 3.2
    - 024_1_0: Pump speed relay 4.1
    - 025_3_0: Timer 4.1
    - 028_1_0: Pump speed relay 4.2
    - 029_3_0: Timer 4.2
    - 032_1_0: Pump speed relay 5.1
    - 033_3_0: Timer 5.1
    - 036_1_0: Pump speed relay 5.2
    - 037_3_0: Timer 5.2
    - 040_1_0: SensorOutputType1
    - 041_1_0: SensorOutputType2
    - 042_1_0: SensorOutputType3
    - 043_1_0: SensorOutputType4
    - 044_1_0: SensorOutputType5
    - 045_1_0: SensorOutputType6
- 00_6651_7E11_10_0200: DeltaSol MX [Regler] => EM #1
    - 000_1_0: Pump speed relay 1.1
    - 001_3_0: Timer 1.1
    - 004_1_0: Pump speed relay 1.2
    - 005_3_0: Timer 1.2
    - 008_1_0: Pump speed relay 2.1
    - 009_3_0: Timer 2.1
    - 012_1_0: Pump speed relay 2.2
    - 013_3_0: Timer 2.2
    - 016_1_0: Pump speed relay 3.1
    - 017_3_0: Timer 3.1
    - 020_1_0: Pump speed relay 3.2
    - 021_3_0: Timer 3.2
    - 024_1_0: Pump speed relay 4.1
    - 025_3_0: Timer 4.1
    - 028_1_0: Pump speed relay 4.2
    - 029_3_0: Timer 4.2
    - 032_1_0: Pump speed relay 5.1
    - 033_3_0: Timer 5.1
    - 036_1_0: Pump speed relay 5.2
    - 037_3_0: Timer 5.2
    - 040_1_0: SensorOutputType1
    - 041_1_0: SensorOutputType2
    - 042_1_0: SensorOutputType3
    - 043_1_0: SensorOutputType4
    - 044_1_0: SensorOutputType5
    - 045_1_0: SensorOutputType6
- 00_6652_7E11_10_0200: DeltaSol MX [Regler] => EM #2
    - 000_1_0: Pump speed relay 1.1
    - 001_3_0: Timer 1.1
    - 004_1_0: Pump speed relay 1.2
    - 005_3_0: Timer 1.2
    - 008_1_0: Pump speed relay 2.1
    - 009_3_0: Timer 2.1
    - 012_1_0: Pump speed relay 2.2
    - 013_3_0: Timer 2.2
    - 016_1_0: Pump speed relay 3.1
    - 017_3_0: Timer 3.1
    - 020_1_0: Pump speed relay 3.2
    - 021_3_0: Timer 3.2
    - 024_1_0: Pump speed relay 4.1
    - 025_3_0: Timer 4.1
    - 028_1_0: Pump speed relay 4.2
    - 029_3_0: Timer 4.2
    - 032_1_0: Pump speed relay 5.1
    - 033_3_0: Timer 5.1
    - 036_1_0: Pump speed relay 5.2
    - 037_3_0: Timer 5.2
    - 040_1_0: SensorOutputType1
    - 041_1_0: SensorOutputType2
    - 042_1_0: SensorOutputType3
    - 043_1_0: SensorOutputType4
    - 044_1_0: SensorOutputType5
    - 045_1_0: SensorOutputType6
- 00_6653_7E11_10_0200: DeltaSol MX [Regler] => EM #3
    - 000_1_0: Pump speed relay 1.1
    - 001_3_0: Timer 1.1
    - 004_1_0: Pump speed relay 1.2
    - 005_3_0: Timer 1.2
    - 008_1_0: Pump speed relay 2.1
    - 009_3_0: Timer 2.1
    - 012_1_0: Pump speed relay 2.2
    - 013_3_0: Timer 2.2
    - 016_1_0: Pump speed relay 3.1
    - 017_3_0: Timer 3.1
    - 020_1_0: Pump speed relay 3.2
    - 021_3_0: Timer 3.2
    - 024_1_0: Pump speed relay 4.1
    - 025_3_0: Timer 4.1
    - 028_1_0: Pump speed relay 4.2
    - 029_3_0: Timer 4.2
    - 032_1_0: Pump speed relay 5.1
    - 033_3_0: Timer 5.1
    - 036_1_0: Pump speed relay 5.2
    - 037_3_0: Timer 5.2
    - 040_1_0: SensorOutputType1
    - 041_1_0: SensorOutputType2
    - 042_1_0: SensorOutputType3
    - 043_1_0: SensorOutputType4
    - 044_1_0: SensorOutputType5
    - 045_1_0: SensorOutputType6
Storing data set for timestamp 2017-03-19 07:47:09.092632 UTC
Storing data set for timestamp 2017-03-19 07:48:00.090876 UTC
Storing data set for timestamp 2017-03-19 07:49:01.633169 UTC
Storing data set for timestamp 2017-03-19 07:50:01.636954 UTC
Storing data set for timestamp 2017-03-19 07:51:00.638965 UTC
Storing data set for timestamp 2017-03-19 07:52:00.635063 UTC
Storing data set for timestamp 2017-03-19 07:53:00.112437 UTC
Storing data set for timestamp 2017-03-19 07:54:00.119384 UTC
Storing data set for timestamp 2017-03-19 07:55:00.116181 UTC
Storing data set for timestamp 2017-03-19 07:56:01.656859 UTC
Storing data set for timestamp 2017-03-19 07:57:01.665480 UTC
Storing data set for timestamp 2017-03-19 07:58:00.652715 UTC
Storing data set for timestamp 2017-03-19 07:59:00.661004 UTC
Storing data set for timestamp 2017-03-19 08:00:00.136048 UTC
Storing data set for timestamp 2017-03-19 08:01:00.130741 UTC
Storing data set for timestamp 2017-03-19 08:02:01.670224 UTC
...
```

Im Folgenden wird auf die einzelnen Ausgaben eingegangen.

#### `Connecting to database...`

Die Datenbankverbindung wird hergestellt. Eventuell direkt danach angezeigte
Fehler hängen mit hoher Wahrscheinlichkeit mit der Datenbank-Konfiguration
oder deren Erreichbarkeit zusammen.


#### `Finding last record...`

Die darauf folgende Zeile `last_timestamp = ...` zeigt an, welchen Zeitstempel
der jüngste Eintrag in der Datenbank trägt. Diese Zeile dient im Moment nur
Informationszwecken.


#### `Process field configuration...`

Während dieses Schritts werden die `[[fields]]`-Sektionen der Konfigurationsdatei
überprüft. Mögliche Fehler werden direkt im Anschluss an diese Zeile ausgegeben.


#### `Connecting to VBus...`

Die Verbindung zum VBus wird hergestellt. Wenn die in der Konfiguration
angegebene serielle Schnitttstelle nicht gefunden werden kann, wird ein
Fehler ausgegeben.


#### `Data set changed, waiting for it to stabilize again...`

Normalerweise werden über den VBus immer zyklisch die selben Paket-Typen
empfangen. Die Menge und Reihenfolge ist dabei aber Regler- und
Firmware-abhängig und nicht vorher bekannt.

Mit der "Data set stabilization" verschafft sich die Anwendung erst einmal einen
Überblick über unterschiedlichen VBus-Paket-Typen, die empfangen werden. Immer,
wenn ein neuer Paket-Typ emfangen wurde, der seit dem Programmstart noch nicht
vorher gesehen wurde, beginnt die Stabilisierungsphase erneut.


#### `Data set stabilizing at ...%`

Je mehr Pakete empfangen werden, deren Paket-Typ bekannt ist, umso höher steigt
der Prozentwert der Stabilisierung. Wird die 100%-Marke erreicht, wechselt
die Anwendung in die Datenaufzeichnung mit der Meldung:


#### `Data set stabilized`

Diese Meldung wird gefolgt von einer Liste der empfangenen VBus-Datenpunkte.
Ein Auszug aus der Beispielausgabe oben:

```
- 00_0010_7E11_10_0100: DeltaSol MX [Regler]
    - 000_2_0: Temperature sensor 1
    - 002_2_0: Temperature sensor 2
    - 004_2_0: Temperature sensor 3
```

Die nicht eingerückten Zeilen sind VBus-Pakete mit ihren `packet_id`s (in diesem
Fall `00_0010_7E11_10_0100`), die eingerückten Zeilen sind VBus-Datenpunkte
innerhalb des darüberliegenden Pakets (hier zum Beispiel `004_2_0` für die
Temperatur an Sensor 3).

Diese Liste ist hilfreich bei der Definition der `[[fields]]`-Sektionen in der
Konfigurationsdatei. Soll z.B. die Temperatur an Sensor 3 in die Datenbank-Spalte
`temp_bypass` abgelegt werden, muss die folgende Feld-Definition in der
Konfiguratiosdatei angegeben werden:

```toml
[[fields]]
column = "temp_bypass"
packet_id = "00_0010_7E11_10_0100"
field_id = "004_2_0"
```


#### `Storing data set for timestamp ...`

Diese Zeile wird nach jedem abgelaufenen Log-Interval angezeigt. Kurz danach
wird die entsprechende Zeile in der Tabelle erzeugt und mit Werten befüllt.

In diesem Zustand verharrt die Anwendung bis zum Ende.
