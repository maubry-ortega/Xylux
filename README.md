# Xylux IDE: Un Entorno de Desarrollo Completo para Juegos y Aplicaciones con Alux

**Versión:** 1.1  
**Fecha:** 2025-08-12  
**Licencia:** Licencia Propia de Xylux  

**Xylux IDE** es un entorno de desarrollo integrado (IDE) ligero y modular, escrito en **Rust**, diseñado específicamente para desarrolladores de videojuegos y aplicaciones que utilizan el motor **Xylux** y el lenguaje de scripting **Alux**. Este IDE ofrece herramientas avanzadas para scripting, depuración, compilación y ejecución, optimizadas para flujos de trabajo de desarrollo de juegos en Rust y Alux, con un enfoque en rendimiento y personalización.

## Características Principales

- **Editor optimizado:** Núcleo ligero con soporte UTF-8, resaltado de sintaxis para Alux y Rust, y edición eficiente.  
- **Soporte nativo para Alux:** Resaltado de sintaxis, autocompletado, depuración textual y ejecución directa de scripts `.aux` en la AluxVM.  
- **Integración con Xylux:** Gestión de proyectos Xylux, soporte para ECS, shaders (.wgsl), y modo headless para pruebas lógicas.  
- **Terminal integrada:** Ejecuta comandos `xylux-cli` (e.g., `xylux run`, `xylux build --target wasm`) desde el IDE.  
- **Depuración avanzada:** Breakpoints en scripts Alux, inspección de variables (e.g., `vec3`, `option<T>`), y soporte para estados de corutinas (`task<T>`).  
- **Hot-reload:** Recarga scripts Alux sin reiniciar el motor, con verificación de bytecode (`.auxc`) para seguridad.  
- **Soporte WebAssembly:** Compila y depura proyectos exportados a navegadores.  
- **Personalización:** Configuración modular vía `config.ini` y soporte para extensiones (futuro: plugins para LSP y shaders).  
- **Interfaz escalable:** Preparado para migrar a GUI completa con egui o iced, manteniendo un diseño minimalista.  

## Instalación

### Con Cargo
Instala Xylux IDE directamente desde crates.io:

```bash
cargo install xylux-ide
```

Esto instala el binario `xylux-ide`, el compilador de Alux (`alux-compiler`) y la máquina virtual (`alux-vm`).

### Otras plataformas
- **Linux/Windows/macOS:** Binarios precompilados disponibles en `https://github.com/xylux/xylux-ide/releases`.  
- **Instaladores:** En desarrollo: paquetes DEB/MSI para instalación simplificada.  
- **Móviles:** Futuro soporte para tablets (modo lectura/escritura de scripts Alux).  

### Requisitos
- Rust toolchain (versión estable, >= 1.80).  
- Opcional: `wasm-pack` para exportar a WebAssembly.  
- Espacio en disco: ~200 MB (binario + templates de proyecto).  

## Uso

```bash
xylux-ide                    # Inicia Xylux IDE con un proyecto vacío
xylux-ide <ruta_proyecto>    # Abre un proyecto Xylux o archivo .aux/.rs
xylux-ide --version          # Muestra la versión instalada
xylux-ide --alux-repl        # Inicia un REPL interactivo de Alux
xylux-ide --new <nombre>     # Crea un nuevo proyecto Xylux con estructura estándar
```

### Comandos clave
- **Ctrl+F5:** Compila (Rust + Alux) y ejecuta el proyecto.  
- **Ctrl+Shift+F5:** Ejecuta en modo headless (pruebas lógicas sin render).  
- **Ctrl+B:** Compila scripts Alux a bytecode (`.aux` → `.auxc`).  
- **Ctrl+D:** Activa modo depuración (breakpoints en Alux).  

### Estructura de proyecto estándar
Un proyecto Xylux típico tiene la siguiente estructura:

```
mi_proyecto/
├── src/             # Código Rust (núcleo del juego)
├── scripts/         # Scripts Alux (.aux)
├── assets/          # Recursos (texturas, modelos)
├── shaders/         # Shaders (.wgsl)
├── Cargo.toml       # Configuración Rust
├── xylux.toml       # Configuración del proyecto Xylux
```

## Configuración

Xylux IDE usa un archivo `config.ini` para personalizar el entorno, ubicado en:

- **Linux/macOS:** `~/.config/xylux-ide/config.ini`  
- **Windows:** `%APPDATA%\XyluxIDE\config.ini`  

Ejemplo de `config.ini`:

```ini
[editor]
theme = "dark"
font = "JetBrains Mono"
font_size = 14
syntax_alux = true
tab_width = 4

[alux]
lsp_enabled = true
hot_reload = true
vm_log_level = "error"

[build]
target = "native"
wasm_opt = true
```

### Opciones destacadas
- `theme`: `dark` (predeterminado), `light`, o temas personalizados.  
- `syntax_alux`: Activa resaltado para Alux (keywords, `vec3`, `entity`).  
- `lsp_enabled`: Integra `alux-lsp` para autocompletado y diagnostics.  
- `hot_reload`: Habilita recarga de scripts Alux en tiempo real.  

## Diferencias con otros IDEs

Xylux IDE es un proyecto completamente independiente, creado desde cero para el ecosistema Xylux y Alux:

- **Enfoque exclusivo en Rust y Alux:** Optimizado para `.rs` (Rust) y `.aux` (Alux), sin soporte genérico para otros lenguajes.  
- **Gestión de proyectos Xylux:** Reconoce estructura de proyectos (`src/`, `scripts/`, `shaders/`), con templates predefinidos.  
- **Terminal integrada:** Ejecuta `xylux-cli` y muestra logs de AluxVM (e.g., errores en `task<T>`).  
- **Herramientas para juegos:** Depuración de corutinas, soporte para shaders, y export a WebAssembly.  
- **Hot-reload y LSP:** Integración con `alux-lsp` y recarga de scripts sin reiniciar.  
- **Escalabilidad a GUI:** Preparado para interfaz gráfica (egui/iced) con paneles, inspector de entidades, y previsualización de shaders.  

## Dependencias

Xylux IDE prioriza un diseño minimalista con dependencias esenciales:

- **Núcleo:** `ropey` (edición de texto), `syntect` (resaltado de sintaxis).  
- **Alux:** `alux-compiler`, `alux-vm`.  
- **Rust:** Integración con `cargo` para compilación.  
- **Plataforma:** `libc` (UNIX), `winapi` (Windows), `unicode-width` (soporte Unicode).  

## Desarrollo y Contribución

El código fuente está disponible en `https://github.com/xylux/xylux-ide`. Para contribuir:

1. Clona el repositorio: `git clone https://github.com/xylux/xylux-ide`.  
2. Crea un branch: `git checkout -b feature/<nombre>`.  
3. Ejecuta tests: `cargo test`.  
4. Formatea código: `cargo fmt` y verifica con `cargo clippy`.  
5. Envía un PR con descripción clara.

### Actualización de código
Envía snippets de código fuente (e.g., `src/main.rs`) para actualizaciones específicas, como integrar soporte para Alux o modificar funciones para el motor Xylux. Usa el mismo `artifact_id` para actualizaciones de archivos previos.

## Licencia

Xylux IDE está licenciado bajo una **Licencia Propia de Xylux**, de uso exclusivo y propietario. Cualquier uso, distribución o modificación debe:

- Atribuir explícitamente al equipo de Xylux como creadores originales.  
- Indicar la fuente del proyecto (`https://github.com/xylux/xylux-ide`) en cualquier fork o derivado.  
- Prohibir el uso no autorizado sin permiso explícito del equipo Xylux.  

Ver `LICENSE.md` en el repositorio para detalles completos.

## Roadmap

- **Fase 0 (1-2 semanas):** Creación del IDE, soporte básico para Alux (sintaxis, ejecución).  
- **Fase 1 (3-6 semanas):** Compilación Rust+Alux, consola embebida, gestión de proyectos.  
- **Fase 2 (7-12 semanas):** Depuración textual, hot-reload, integración `xylux-cli`.  
- **Fase 3 (12+ semanas):** GUI completa (egui/iced), perfilador, editor de shaders, soporte WASM avanzado.  

## Contacto y Soporte

- **Issues:** Reporta bugs en `https://github.com/xylux/xylux-ide/issues`.  
- **Documentación:** Consulta `docs/Design-v0.2.md` y `docs/Alux-Syntax-v0.2.md` en el repositorio.  
- **Comunidad:** Únete a nuestro canal en `https://discord.gg/xylux` (en desarrollo).  

**¡Construye desde las coordenadas XY y da luz a tus juegos con Xylux IDE y Alux!**