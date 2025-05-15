
set -e

echo "🔨 Compilando en modo release..."
cargo build --release

echo "📦 Instalando binario en /usr/local/bin/"
sudo cp target/release/global_pomodoro /usr/local/bin/

echo "🔊 Copiando archivos de sonido..."
sudo mkdir -p /usr/share/global_pomodoro/sounds
sudo cp src/sounds/* /usr/share/global_pomodoro/sounds/

echo "📁 Restableciendo archivos de configuración..."
rm -rf ~/.config/global_pomodoro
mkdir -p ~/.config/global_pomodoro
cp blocked_sites.json ~/.config/global_pomodoro/
cp pomodoro_config.json ~/.config/global_pomodoro/

echo "✅ Instalación completa."
echo "ℹ️  La configuración ha sido restablecida a los valores por defecto."
