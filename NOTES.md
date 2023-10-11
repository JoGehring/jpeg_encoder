# NOTIZEN

## Datenstrukturen

Bilder werden in einem struct abgebildet, das ein dreidimensionales Feld enthält:

```pseudocode
[
    [[R1, G1, B1], [R2, G2, B2], ...], // Reihe 1
    [[R1, G1, B1], [R2, G2, B2], ...], // Reihe 2
]
```

Außerdem enthält dieses struct den Farbraum, der als Enum definiert sein müsste.

Begründung: Das ermöglicht einfach-ishe Manipulation eines Farbkanals und guten Zugriff auf "Schrittweiten"

## Bibliotheken

[nalgebra](https://www.nalgebra.org/docs/user_guide/getting_started) für LinAlg
