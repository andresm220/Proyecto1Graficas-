# Proyecto 1 — Gráficas (Rust)

Aplicación/juego simple para el curso de Gráficas, implementado en **Rust**. 
El objetivo es renderizar un nivel completo y jugable, con colisiones, distintas texturas y contador de FPS.

Link al video mostrando el funcionamiento:https://youtu.be/Mf49h3xLqrU 



## Características (objetivos del proyecto)
- Movimiento del jugador (adelante/atrás) y **rotación**.
- **Colisiones** contra paredes: no se atraviesan.
- **Colores/texturas** distintas por pared/material.
- **Contador de FPS** en pantalla.
- Arquitectura simple y legible para extender niveles.




## Requisitos

- **Rust** (toolchain estable): <https://www.rust-lang.org/tools/install>
- `cargo` (se instala con Rust)


---

## Cómo correr

```bash
# Clona el repo
git clone https://github.com/andresm220/Proyecto1Graficas-.git
cd Proyecto1Graficas-/maze_gen
# Compila y ejecuta en modo debug
cargo run

# Recomendado para mejor rendimiento
cargo run --release



