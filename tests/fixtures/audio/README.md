# Audio-Testdateien

Die kurzen synthetischen Sinustöne wurden mit FFmpeg erzeugt und enthalten keine Sprachaufnahmen. Die Tests benötigen FFmpeg nicht; sie lesen diese Dateien und dekodieren die erzeugten MP3-Dateien mit Symphonia.

- `mono.m4a`: AAC, 44,1 kHz, Mono, 0,12 s.
- `stereo.mp4`: AAC, 48 kHz, zwei unterschiedliche Töne, 1,25 s.
- `mono.ogg`: Vorbis, 48 kHz, Mono, 1,25 s.
- `surround.m4a`: AAC, 48 kHz, sechs Kanäle; prüft die Fehlermeldung für nicht unterstützten Mehrkanalton.
