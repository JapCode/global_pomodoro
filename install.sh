set -e

echo "🔨 Compilando en modo release..."
cargo build --release

echo "📦 Instalando binario en /usr/local/bin/"
sudo cp target/release/global_pomodoro /usr/local/bin/

echo "📁 Copiando archivos de configuración base y recursos..."
sudo mkdir -p /usr/share/global_pomodoro
sudo cp sounds/* /usr/share/global_pomodoro/

sudo mkdir -p /etc/global_pomodoro
sudo cp blocked_sites.json /etc/global_pomodoro/
sudo cp pomodoro_config.json /etc/global_pomodoro/


echo "✅ Instalación completa."
echo "ℹ️  La configuración editable se copiará a ~/.config/global_pomodoro/ al primer uso."
