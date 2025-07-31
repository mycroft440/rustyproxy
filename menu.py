
import os
from rich.console import Console
from rich.theme import Theme

# Configuração do console com tema personalizado
custom_theme = Theme({
    "info": "dim cyan",
    "warning": "magenta",
    "danger": "bold red",
    "success": "bold green",
    "menu": "bold blue",
    "option": "cyan",
    "header": "bold yellow on blue",
    "border": "blue",
})
console = Console(theme=custom_theme)

PORTS_FILE = "ports.txt"

def clear_screen():
    os.system('cls' if os.name == 'nt' else 'clear')

def get_active_ports():
    ports = []
    if os.path.exists(PORTS_FILE):
        with open(PORTS_FILE, 'r') as f:
            ports = [line.strip() for line in f if line.strip()]
    return ports

def save_ports(ports):
    with open(PORTS_FILE, 'w') as f:
        for port in ports:
            f.write(f"{port}\n")

def is_port_in_use(port):
    # Esta é uma simulação. Em um ambiente real, você usaria ferramentas de sistema.
    # Por enquanto, apenas verifica se a porta está na nossa lista de portas ativas.
    return str(port) in get_active_ports()

def add_proxy_port(port, status="@RustyProxy"):
    if is_port_in_use(port):
        console.print(f"[warning]A porta {port} já está em uso.[/warning]")
        return

    ports = get_active_ports()
    ports.append(str(port))
    save_ports(ports)
    console.print(f"[success]Porta {port} aberta com sucesso.[/success]")

def del_proxy_port(port):
    ports = get_active_ports()
    if str(port) in ports:
        ports.remove(str(port))
        save_ports(ports)
        console.print(f"[success]Porta {port} fechada com sucesso.[/success]")
    else:
        console.print(f"[warning]A porta {port} não está ativa.[/warning]")

def show_menu():
    clear_screen()
    console.print("[header]===== Adaptação de Multiflow Manager======[/header]")
    console.print("[border]------------------------------------------------[/border]")
    console.print("[menu]|                  RUSTY PROXY                  |[/menu]")
    console.print("[border]------------------------------------------------[/border]")
    
    active_ports = get_active_ports()
    if not active_ports:
        console.print("[option]| Portas(s): nenhuma                                |[/option]")
    else:
        ports_str = ", ".join(active_ports)
        console.print(f"[option]| Portas(s): {ports_str:<34} |[/option]")

    console.print("[border]------------------------------------------------[/border]")
    console.print("[option]| 1 - Abrir Porta                               |[/option]")
    console.print("[option]| 2 - Fechar Porta                              |[/option]")
    console.print("[option]| 0 - Sair                                      |[/option]")
    console.print("[border]------------------------------------------------[/border]")
    console.print()

    option = console.input("[info] --> Selecione uma opção: [/info]")
    return option

def main():
    # Cria o arquivo de portas se não existir
    if not os.path.exists(PORTS_FILE):
        with open(PORTS_FILE, 'w') as f:
            pass

    while True:
        option = show_menu()

        if option == '1':
            port = console.input("[info]Digite a porta: [/info]")
            while not port.isdigit():
                console.print("[danger]Digite uma porta válida.[/danger]")
                port = console.input("[info]Digite a porta: [/info]")
            status = console.input("[info]Digite o status de conexão (deixe vazio para o padrão): [/info]")
            add_proxy_port(int(port), status if status else "@RustyProxy")
            console.input("[info]> Porta ativada com sucesso. Pressione ENTER para voltar ao menu.[/info]")
        elif option == '2':
            port = console.input("[info]Digite a porta: [/info]")
            while not port.isdigit():
                console.print("[danger]Digite uma porta válida.[/danger]")
                port = console.input("[info]Digite a porta: [/info]")
            del_proxy_port(int(port))
            console.input("[info]> Porta desativada com sucesso. Pressione ENTER para voltar ao menu.[/info]")
        elif option == '0':
            console.print("[info]Saindo...[/info]")
            break
        else:
            console.print("[danger]Opção inválida. Pressione ENTER para voltar ao menu.[/danger]")
            console.input()

if __name__ == "__main__":
    main()


