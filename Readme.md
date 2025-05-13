# Pomodoro: Cancelación de Operación

## Cómo cancelar la operación

### Opción 1: Presionar `Ctrl+C`

Puedes interrumpir el programa de manera inmediata presionando `Ctrl+C` en la terminal. Esto detendrá la ejecución del temporizador y desbloqueará los sitios web bloqueados.

---

### Opción 2: Archivo `cancel_pomodoro.txt`

El programa también permite cancelar el Pomodoro creando un archivo especial llamado `cancel_pomodoro.txt`.

#### ¿Qué hace el archivo?

- Si el programa detecta que existe este archivo, detiene el Pomodoro inmediatamente.
- Desbloquea todos los sitios bloqueados.
- Guarda el estado del Pomodoro como "cancelado".

#### ¿Cómo crear el archivo?

Utiliza el siguiente comando en la terminal para crear el archivo de cancelación:

```bash
touch cancel_pomodoro.txt
```

#### Limpieza del archivo de cancelación:

Puedes eliminar manualmente el archivo cancel_pomodoro.txt después de cancelar:

```bash
rm cancel_pomodoro.txt
```
