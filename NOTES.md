# NOTIZEN

## Datenstrukturen

Bilder werden in einem struct abgebildet, das drei Felder enthält. Jedes davon ist ein zweidimensionales Array, das die Bilddaten für jeweils einen Kanal beinhaltet.

Für jeden Kanal wird außerdem gespeichert, in welchem Ausmaß er heruntergerechnet wurde.

Außerdem enthält dieses struct den Farbraum, der als Enum definiert sein müsste (TODO).

Begründung: Das ermöglicht einfach-ishe Manipulation eines Farbkanals und guten Zugriff auf "Schrittweiten". Außerdem ist es möglich, einzelne Kanäle unabhängig voneinander herunterzurechnen.

Einzelne R/G/B-Werte werden als vorzeichenlose 16-Bit-Integer (u16) gespeichert, mit einer Werterange von 0 bis 65536.

## Bibliotheken

[nalgebra](https://www.nalgebra.org/docs/user_guide/getting_started) für LinAlg
