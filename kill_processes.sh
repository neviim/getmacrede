#!/bin/bash

# Script para listar e encerrar processos do getmacrede
# Autor: Script gerado para gerenciar processos da aplicação

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Função para listar processos
list_processes() {
    echo -e "${CYAN}=== Processos do getmacrede ===${NC}\n"

    # Busca por processos relacionados
    PIDS=$(ps aux | grep -E "(getmacrede|rust_out)" | grep -v grep | grep -v "$0")

    if [ -z "$PIDS" ]; then
        echo -e "${YELLOW}Nenhum processo encontrado.${NC}"
        return 1
    fi

    echo -e "${GREEN}Processos encontrados:${NC}"
    echo ""
    ps aux | grep -E "(getmacrede|rust_out)" | grep -v grep | grep -v "$0" | \
        awk '{printf "%-8s %-8s %-6s %s\n", "PID:", $2, "USER:", $1; printf "CMD: "; for(i=11;i<=NF;i++) printf "%s ", $i; printf "\n\n"}'

    return 0
}

# Função para matar processos
kill_processes() {
    local SIGNAL=$1

    PIDS=$(ps aux | grep -E "(getmacrede|rust_out)" | grep -v grep | grep -v "$0" | awk '{print $2}')

    if [ -z "$PIDS" ]; then
        echo -e "${YELLOW}Nenhum processo para encerrar.${NC}"
        return 1
    fi

    echo -e "${YELLOW}Encerrando processos...${NC}"

    for PID in $PIDS; do
        if [ "$SIGNAL" == "FORCE" ]; then
            echo -e "Enviando SIGKILL (força) para PID ${RED}$PID${NC}"
            sudo kill -9 $PID 2>/dev/null
        else
            echo -e "Enviando SIGTERM (normal) para PID ${RED}$PID${NC}"
            sudo kill $PID 2>/dev/null
        fi
    done

    sleep 1

    # Verifica se ainda há processos
    REMAINING=$(ps aux | grep -E "(getmacrede|rust_out)" | grep -v grep | grep -v "$0")
    if [ -z "$REMAINING" ]; then
        echo -e "\n${GREEN}✓ Todos os processos foram encerrados com sucesso!${NC}"
    else
        echo -e "\n${YELLOW}⚠ Alguns processos ainda estão em execução:${NC}"
        list_processes
        echo -e "\n${YELLOW}Tente usar a opção --force para encerrar à força.${NC}"
    fi
}

# Menu interativo
interactive_menu() {
    while true; do
        echo -e "\n${CYAN}=== Menu de Gerenciamento de Processos ===${NC}"
        echo "1) Listar processos"
        echo "2) Encerrar processos (SIGTERM)"
        echo "3) Encerrar processos FORÇA (SIGKILL)"
        echo "4) Sair"
        echo -n "Escolha uma opção: "
        read -r OPTION

        case $OPTION in
            1)
                list_processes
                ;;
            2)
                kill_processes "NORMAL"
                ;;
            3)
                kill_processes "FORCE"
                ;;
            4)
                echo -e "${GREEN}Saindo...${NC}"
                exit 0
                ;;
            *)
                echo -e "${RED}Opção inválida!${NC}"
                ;;
        esac
    done
}

# Parse de argumentos
case "$1" in
    -l|--list)
        list_processes
        ;;
    -k|--kill)
        kill_processes "NORMAL"
        ;;
    -f|--force)
        kill_processes "FORCE"
        ;;
    -h|--help)
        echo "Uso: $0 [OPÇÃO]"
        echo ""
        echo "Opções:"
        echo "  -l, --list     Lista todos os processos do getmacrede"
        echo "  -k, --kill     Encerra os processos (SIGTERM)"
        echo "  -f, --force    Encerra os processos à força (SIGKILL)"
        echo "  -h, --help     Exibe esta mensagem de ajuda"
        echo "  (sem opção)    Abre menu interativo"
        echo ""
        echo "Exemplos:"
        echo "  $0 --list      # Lista processos"
        echo "  $0 --kill      # Encerra normalmente"
        echo "  $0 --force     # Encerra à força"
        ;;
    "")
        interactive_menu
        ;;
    *)
        echo -e "${RED}Opção inválida: $1${NC}"
        echo "Use --help para ver as opções disponíveis"
        exit 1
        ;;
esac
