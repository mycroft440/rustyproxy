#!/bin/bash
# rustyproxy Installer

TOTAL_STEPS=11
CURRENT_STEP=0

show_progress() {
    PERCENT=$((CURRENT_STEP * 100 / TOTAL_STEPS))
    echo "Progresso: [${PERCENT}%] - $1"
}

error_exit() {
    echo -e "\nErro: $1"
    exit 1
}

increment_step() {
    CURRENT_STEP=$((CURRENT_STEP + 1))
}

if [ "$EUID" -ne 0 ]; then
    error_exit "EXECUTE COMO ROOT"
else
    clear
    show_progress "Atualizando repositorios..."
    export DEBIAN_FRONTEND=noninteractive
    apt update -y > /dev/null 2>&1 || error_exit "Falha ao atualizar os repositorios"
    increment_step

    # ---->>>> Verificação do sistema
    show_progress "Verificando o sistema..."
    if ! command -v lsb_release &> /dev/null; then
        apt install lsb-release -y > /dev/null 2>&1 || error_exit "Falha ao instalar lsb-release"
    fi
    increment_step

    # ---->>>> Verificação do sistema
    OS_NAME=$(lsb_release -is)
    VERSION=$(lsb_release -rs)

    case $OS_NAME in
        Ubuntu)
            case $VERSION in
                24.*|22.*|20.*|18.*)
                    show_progress "Sistema Ubuntu suportado, continuando..."
                    ;;
                *)
                    error_exit "Versão do Ubuntu não suportada. Use 18, 20, 22 ou 24."
                    ;;
            esac
            ;;
        Debian)
            case $VERSION in
                12*|11*|10*|9*)
                    show_progress "Sistema Debian suportado, continuando..."
                    ;;
                *)
                    error_exit "Versão do Debian não suportada. Use 9, 10, 11 ou 12."
                    ;;
            esac
            ;;
        *)
            error_exit "Sistema não suportado. Use Ubuntu ou Debian."
            ;;
    esac
    increment_step

    # ---->>>> Instalação de pacotes requisitos e atualização do sistema
    show_progress "Atualizando o sistema..."
    apt upgrade -y > /dev/null 2>&1 || error_exit "Falha ao atualizar o sistema"
    apt-get install curl build-essential git -y > /dev/null 2>&1 || error_exit "Falha ao instalar pacotes"
    increment_step

    # ---->>>> Criando o diretório do script
    show_progress "Criando diretorio /opt/rustyproxy..."
    mkdir -p /opt/rustyproxy > /dev/null 2>&1
    increment_step

    # ---->>>> Instalar python3 e pip (Movido para antes da compilação)
    show_progress "Instalando python3 e pip..."
    apt-get install python3 python3-pip -y > /dev/null 2>&1 || error_exit "Falha ao instalar python3 e pip"
    increment_step

    # ---->>>> Instalar a biblioteca rich (Movido para antes da compilação)
    show_progress "Instalando a biblioteca rich..."
    apt-get install python3-rich -y > /dev/null 2>&1 || error_exit "Falha ao instalar a biblioteca rich"
    increment_step

    # ---->>>> Instalar rust
    show_progress "Instalando Rust..."
    if ! command -v rustc &> /dev/null; then
        curl --proto =https --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y > /dev/null 2>&1 || error_exit "Falha ao instalar Rust"
        source "$HOME/.cargo/env"
    fi
    increment_step

    # ---->>>> Instalar o RustyProxy
    show_progress "Compilando RustyProxy, isso pode levar algum tempo dependendo da maquina..."

    if [ -d "/root/RustyProxyOnly" ]; then
        rm -rf /root/RustyProxyOnly
    fi

    git clone --branch "main" https://github.com/mycroft440/rustyproxy.git /root/RustyProxyOnly > /dev/null 2>&1 || error_exit "Falha ao clonar rustyproxy"
    mv /root/RustyProxyOnly/menu.py /opt/rustyproxy/menu.py
    cd /root/RustyProxyOnly/RustyProxy
    cargo build --release --jobs $(nproc) > /dev/null 2>&1 || error_exit "Falha ao compilar rustyproxy"
    mv ./target/release/RustyProxy /opt/rustyproxy/proxy
    increment_step

    # ---->>>> Configuração de permissões
    show_progress "Configurando permissões..."
    chmod +x /opt/rustyproxy/proxy
    chmod +x /opt/rustyproxy/menu.py
    echo -e "#!/usr/bin/python3\nimport os\nimport sys\nsys.path.append(\"/opt/rustyproxy\")\nos.execv(\"/usr/bin/python3\", [\"python3\", \"/opt/rustyproxy/menu.py\"] + sys.argv[1:])" | sudo tee /usr/local/bin/rustyproxy > /dev/null
    chmod +x /usr/local/bin/rustyproxy
    increment_step

    # ---->>>> Limpeza
    rm -rf /root/RustyProxyOnly/
    increment_step

    # ---->>>> Instalação finalizada :)
    echo "Instalação concluída com sucesso. Digite 'rustyproxy' para acessar o menu."
fi