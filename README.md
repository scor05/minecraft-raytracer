# Minecraft Raytracer 
---
En este repositorio se encuentra mi implementación en Rust (y parte con el código visto en clase) de un raytracer que renderiza 
una escena usando cubos de minecraft, la cual muestra un mini castillo en donde el jugador vivió adyacente a una isla de
con un portal al Nether que rompió despues de haber conquistado dicha dimensión (mini lore dump).

La escena está compuesta de dos islas principales (las cuales fueron generadas procedimentalmente con el algorítmo de Perlin en `./src/procedural.rs`, asímismo como las hojas del árbol): una hecha de tierra y grama y otra hecha de netherrack. En la primera de estas está un castillo con una bandera en su cima, un árbol a su derecha, un mini río (con reflexiones y refracciones en el agua) en frente de él, un cofre y varios barriles donde se almacenaban supplies varias. Dentro del castillo hay dos linternas que emiten luz y un bloque de oro que el jugador tomó como trofeo del Nether.

Esa isla está conectada a la otra mediante un puente pequeño hecho de oak planks de 4 bloques de longitud, en la cual se encuentra un portal al nether roto en su esquina superior. Encima del portal quedaron dos bloques de oro puestos por el jugador y una plataforma de piedra para simbolizar que este portal ya funciona solo como un monumento. 

## Cómo ejecutar y Detalles
---
El código está diseñado para que se pueda ejecutar desde la raíz del proyecto y con respecto a ella leer todos los archivos de texturas
en `./assets/textures/`. Para ejecutar el código, el siguiente comando abrirá una ventana pequeña donde ya se puede ver la escena:

```bash
cargo run
```

Un detalle que es de notar es que para optimizar un poco más el código (debido a que mi laptop estaba sobrecalentándose al mover la cámara y mi uso de CPU estaba llegando casi al 100%) agregué un 95% de probabilidad de que cada pixel dado no se renderice mientras que está en movimiento 
la cámara, lo cual si se desea se puede quitar poniendo la constante `PIXEL_OPTIMIZATION` en `./src/main.rs:28` como falsa. El código ya está 
optimizado con paralelismo dividiendo las filas de pixeles del framebuffer entre los núcleos disponibles del CPU con `thread::available_parallelism`, 
los cuales cada uno en su scope calculan los rayos y colores correspondientes a sus pixeles y luego se combinan al ya procesar todo el frame, 
asímismo no renderizando nada nuevo si no se ha movido la cámara y solo dejando la última vista que se renderizó. También si se desea, se puede 
ajustar la resolución del framebuffer cambiando las variables `WINDOW_WIDTH` y `WINDOW_HEIGHT` en `./src/main.rs:28-29` para tener una imagen de mejor calidad.

## Controles
---
Para mover la cámara, se puede usar WASD para moverla en el plano horizontal de lado a lado y de enfrente a atrás; con LCTRL funcionando para
mover la cámara hacia abajo y SPACEBAR para arriba. Las flechas del teclado controlan la rotación de la cámara y se puede hacer zoom con Q y 
zoomout con E.

## Demostración
---
Una demostración del código en funcionamiento se puede ver en [este video de youtube](https://youtu.be/Q2008-UGgrI). El video fue grabado con 
la resolución del framebuffer puesta a 400x300 y escalada con OBS a 1920x1080, pues tener OBS grabando junto con el raytracer causaba que mi laptop 
tuviera pausas demasiado grandes entre frames a resoluciones más altas, por lo que le tuve que bajar a dicha resolución.
