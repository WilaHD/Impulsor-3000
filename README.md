<div align="center">
<h1> Impulsor-3000<img src="imgs/logo.png" width="50" align="center" style="padding-left:5; padding-bottom:15" /></h1>
</div>

![GitHub Release](https://img.shields.io/github/v/release/WilaHD/Impulsor-3000?label=release%20version)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/WilaHD/Impulsor-3000/.github%2Fworkflows%2Frelease.yml?label=build%20release)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/WilaHD/Impulsor-3000/.github%2Fworkflows%2Frust.yml?label=build%20rust)



Ein Werkzeug zur Umwandlung der eingereichten Impuls PDF-Dateien in Bilder und HTML-Vorlagen für den täglichen Impuls der Kirchgemeinden im Striegistal. Jetzt auch mit der Möglichkeit zur Umwandlung von WhatsApp-Sprachnachrichten in MP3-Dateien.

- [:earth_africa: Webseite der Kirchgemeinden im Striegistal und der Zschopau](https://kirchen-zschopau-striegis.de/)
- [:newspaper: Telegram-Bot für den täglichen Impuls](https://t.me/tagesimpuls_bot)
- [:newspaper: WhatsApp-Kanal für den täglichen Impuls](https://whatsapp.com/channel/0029VarVA8AIyPtYcBrnFG1p)

## Neuigkeiten
Alle Neuigkeiten und Veränderungen zu einer neuen Hauptversion des Programms können dem [Änderungsprotokoll](./CHANGELOG.md) entnommen werden.


## Installation
Aktuell nur für gängige Windows 10 und 11 (x64) sowie Linux (x64) und MacOS (arm64) Systeme verfügbar.
Versionen für weitere Plattformen sind nicht verfügbar.

[Download](https://github.com/wilahd/Impulsor-3000/releases/latest/) &larr; Impulsor-3000 hier im Abschnitt "Assets" plattformspezifisch herunterladen.

<!--
[![GitHub Downloads (specific asset, latest release)](https://img.shields.io/github/downloads/WilaHD/Impulsor-3000/latest/impulsor3000_setup.exe%20?label=Download%20Windows&color=blue)
](https://github.com/wilahd/Impulsor-3000/releases/latest/download/impulsor3000_setup.exe)
&larr; Installationsprogramm für Windows hier herunterladen.

[![GitHub Downloads (specific asset, latest release)](https://img.shields.io/github/downloads/wilahd/Impulsor-3000/latest/Impulsor-3000_x86-64.AppImage?label=Download%20Linux&color=blue)](https://github.com/wilahd/Impulsor-3000/releases/latest/download/Impulsor-3000_x86-64.AppImage)
&larr; AppImage für Linux hier herunterladen.

[![GitHub Downloads (specific asset, latest release)](https://img.shields.io/github/downloads/wilahd/Impulsor-3000/latest/Impulsor-3000-macos-arm64.zip?label=Download%20MacOS&color=blue)](https://github.com/wilahd/Impulsor-3000/releases/latest/download/Impulsor-3000-macos-arm64.zip)
&larr; App für MacOS hier herunterladen und Zipp-Datei entpacken.
-->

## PDF-Konvertierung

Die erste Seite wird mit `hayro` in ein JPEG mit höchstens etwa 1600 × 2000 Pixeln umgewandelt. Formularwerte werden mit `pdf_oxide` gelesen und an ihren Feldpositionen gezeichnet. Der Zeilenabstand ist in den Einstellungen zwischen 80 und 250 % wählbar; der Standardwert beträgt 133 %. Verfügbares Arial wird bevorzugt; andernfalls verwendet das Programm das mitgelieferte Liberation Sans und zeigt beim Start eine Warnung. Die Ersatzschrift ist unter der [SIL Open Font License 1.1](./fonts/LiberationSans-LICENSE.txt) lizenziert.

PDF-Vorlagen bleiben beim Kopieren der Formularwerte bearbeitbar. Manche PDF-Betrachter aktualisieren ihre Felddarstellung erst beim Öffnen; das erzeugte JPEG verwendet die gespeicherten Werte direkt. Die Bildausgabe unterstützt derzeit linksbündige Textfelder auf nicht gedrehten PDF-Seiten ohne weitere Anmerkungen. Nicht unterstützte Inhalte werden als Fehler gemeldet. Überlange Feldtexte werden im Bild am Feldrand abgeschnitten; ein Warnsymbol zeigt die betroffenen Felder beim Darüberfahren an.

## Audio-Konvertierung

Audiodateien im Format M4A, OGG und MP4 werden mit Symphonia dekodiert und mit `rusty_mp3` als MP3 gespeichert. Mono-Aufnahmen werden als Stereo ausgegeben. Unterstützt werden Mono- und Stereo-Eingaben mit den MP3-Abtastraten 8, 11,025, 12, 16, 22,05, 24, 32, 44,1 und 48 kHz.

## Hinweise für Entwickler

Hier geht es zur [Dokumentation](./docs/README.md).

Für den Build mit den aktuell gesperrten Abhängigkeiten wird Rust 1.92 oder neuer benötigt.
Der MP3-Encoder wird auch im Debug-Build optimiert kompiliert, damit die Audio-Konvertierung bei der Entwicklung schnell bleibt. Der erste Build kann dadurch etwas länger dauern.
