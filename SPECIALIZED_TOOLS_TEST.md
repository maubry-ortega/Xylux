# Specialized Tools Integration Test Guide

## Xylux IDE - Specialized Tools for Rust and Alux

Este documento describe cómo probar la integración de la ventana de herramientas especializadas que hemos añadido al Xylux IDE.

## ¿Qué hemos implementado?

Hemos añadido una **Ventana de Herramientas Especializadas** que proporciona:

### 🦀 Herramientas para Rust:
- **Información de Cargo**: Detalles del proyecto (nombre, versión, edición, características)
- **Dependencias**: Lista de crates y sus versiones
- **Objetivos de Compilación**: Binarios, librerías, ejemplos, tests
- **Resultados de Tests**: Estado de las pruebas ejecutadas
- **Advertencias de Clippy**: Análisis de código estático
- **Configuración de Rustfmt**: Opciones de formateo

### ⚡ Herramientas para Alux:
- **Información del Script**: Metadatos del proyecto Alux
- **Módulos**: Lista de módulos con exportaciones e importaciones
- **Funciones**: Definiciones de funciones con parámetros y tipos
- **Variables**: Estado de variables con tipos y valores
- **Errores de Sintaxis**: Análisis de errores con sugerencias
- **Información de Runtime**: Uso de memoria, tiempo de ejecución, GC

## Cómo probar la integración

### 1. Compilar y ejecutar el IDE

```bash
cd kibi
cargo build --release
./target/release/xylux-ide
```

### 2. Abrir la ventana de herramientas

Una vez que el IDE esté ejecutándose:

1. Ve al menú **Tools** en la barra de menú
2. Haz clic en **🔧 Specialized Tools**
3. Se abrirá una ventana flotante con las herramientas especializadas

### 3. Probar con archivos de ejemplo

Hemos creado dos archivos de ejemplo para demostrar las herramientas:

#### Para Rust:
- Abre el archivo `example.rs` en el IDE
- La ventana de herramientas debería mostrar información de Rust
- Verás datos de Cargo, dependencias, y análisis de código

#### Para Alux:
- Abre el archivo `example.alux` en el IDE  
- La ventana de herramientas debería mostrar información de Alux
- Verás módulos, funciones, variables, y análisis de sintaxis

### 4. Características de la interfaz

La ventana de herramientas incluye:

- **Pestañas intercambiables**: Puedes alternar entre herramientas de Rust y Alux
- **Secciones plegables**: Cada categoría se puede expandir/contraer
- **Actualización automática**: Los datos se actualizan cuando cambias de archivo
- **Iconos descriptivos**: Cada herramienta tiene iconos para fácil identificación

## Estructura del código

### Archivos principales añadidos/modificados:

1. **`src/gui/tools.rs`** - Implementación completa de la ventana de herramientas
2. **`src/gui/mod.rs`** - Exportación del módulo de herramientas
3. **`src/gui/app.rs`** - Integración en la aplicación principal
4. **`src/gui/menu.rs`** - Opción de menú para abrir herramientas
5. **`src/main.rs`** - Correcciones de importaciones

### Características técnicas:

- **Modular**: Cada herramienta está separada en su propia función
- **Configurable**: Se pueden mostrar/ocultar secciones individualmente
- **Extensible**: Fácil añadir nuevas herramientas para otros lenguajes
- **Reactivo**: Responde a cambios en el archivo activo

## Datos de ejemplo mostrados

La implementación actual muestra datos de ejemplo (mock data) para demostrar la funcionalidad:

### Rust (ejemplo):
- Proyecto: "xylux-ide" v0.1.0
- Dependencias: eframe, egui, tokio, etc.
- Edición: 2021
- Características: default, clipboard, network

### Alux (ejemplo):
- Script: "main" v1.0.0
- Módulos: core, math, collections
- Runtime: Información de memoria y GC
- Dependencias: core, math

## Próximos pasos

Para hacer las herramientas completamente funcionales, se necesitaría:

1. **Integración con rust-analyzer** para datos reales de Rust
2. **Parser de Alux** para análisis de sintaxis real
3. **Integración con Cargo** para información de proyecto actual
4. **LSP para Alux** para análisis semántico
5. **Métricas en tiempo real** para información de runtime

## Comandos útiles para desarrollo

```bash
# Compilar en modo debug
cargo build

# Compilar optimizado
cargo build --release

# Verificar sintaxis sin compilar
cargo check

# Ejecutar tests
cargo test

# Formatear código
cargo fmt

# Análisis con clippy
cargo clippy
```

## Estructura de la ventana

```
🔧 Specialized Tools
├── 🦀 Rust Tools
│   ├── 📦 Cargo Information
│   ├── 📚 Dependencies  
│   ├── 🎯 Build Targets
│   ├── 🧪 Test Results
│   └── 📋 Clippy Warnings
└── ⚡ Alux Tools
    ├── 📄 Script Information
    ├── 📦 Modules
    ├── 🔧 Functions
    ├── 💾 Variables
    ├── ❌ Syntax Errors
    └── ⚙️ Runtime Info
```

## Notas de implementación

- La ventana es **no modal** y puede permanecer abierta mientras trabajas
- **Scroll automático** para contenido largo
- **Colores semánticos** para diferentes tipos de información
- **Iconos intuitivos** para cada categoría
- **Responsive design** se adapta al tamaño de la ventana

¡La integración está lista para probar! 🚀