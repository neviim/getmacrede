# Changelog - GetMacRede

## v0.2.0 - 2025-12-03

### 🎨 Sistema de Cores Informativo
Implementado esquema de cores intuitivo que facilita identificação visual instantânea:

**Paleta de Cores:**
| Elemento | Cor | Significado |
|----------|-----|-------------|
| IP | Branco | Dispositivo físico |
| IP | Cyan | VM/Container/Virtual |
| MAC | Verde | Hardware físico (saudável) |
| MAC | Cyan Bold | MAC real de VM (mapeado) |
| MAC | - (cinza) | Vazio (só tem virtual MAC) |
| VIRTUAL MAC | Amarelo | MAC virtual detectado |
| Status Online | Verde | Dispositivo ativo |
| Status Offline | Vermelho | Dispositivo inativo |
| Status Block | Vermelho Bold | Bloqueado |
| Vendor (VM) | Magenta | Virtualização |
| Vendor (Físico) | Branco | Fabricante conhecido |
| Vendor (Vazio) | - (cinza) | Não identificado |
| Hostname | Branco brilhante | Nome configurado |
| Hostname (Vazio) | - (cinza) | Sem nome |

**Legenda Visual:**
- Adicionada legenda no topo da tabela
- Headers em branco bold para destaque
- Sistema de cores semântico e consistente

**Localização**: `src/monitor.rs:281-390`

### 🔧 Correção de IPs Duplicados
**Problema Resolvido:** Sistema criava múltiplas entradas para o mesmo IP quando o MAC mudava (comum em VMs)

**Solução Implementada:**
- Sistema agora rastreia dispositivos por **IP** ao invés de MAC
- Automaticamente remove duplicados ao carregar dados existentes
- Mantém apenas a entrada mais recente para cada IP
- Atualiza MACs dinamicamente quando detecta mudanças

**Antes:**
```
192.168.15.144  8e:9c:33:ee:d0:a6  Offline  Virtual/Private MAC
192.168.15.144  5e:fc:17:1f:61:92  Offline  Virtual/Private MAC
192.168.15.144  5c:01:3b:81:08:80  Offline
```

**Depois:**
```
192.168.15.144                    5c:01:3b:81:08:80  Offline
```

**Localização**: `src/monitor.rs:56-90, 119-192`

### 🎯 Detecção Automática de Virtual MAC
**Funcionalidade:** Sistema detecta automaticamente MACs virtuais e os move para a coluna apropriada

**Características:**
- Detecção baseada em OUI database (vendor lookup)
- Detecção de bit U/L (locally administered addresses)
- Funciona tanto para novos dispositivos quanto para dados existentes
- Integração com mapeamento manual (`mac_mapping.json`)

**Lógica:**
1. Se MAC é virtual E tem mapeamento manual → MAC real na coluna MAC, virtual na coluna VIRTUAL MAC
2. Se MAC é virtual SEM mapeamento → Vazio na coluna MAC, virtual na coluna VIRTUAL MAC
3. Se MAC é físico → MAC na coluna MAC, VIRTUAL MAC vazia

**Benefícios:**
- ✅ Virtual MACs sempre na coluna correta
- ✅ Distinção clara entre MACs reais e virtuais
- ✅ Facilita identificação de VMs/containers
- ✅ Migração automática de dados antigos

**Localização**: `src/monitor.rs:68-72, 112-118, 157-186, 330-341` | `src/vendor.rs:176-191`

### 📊 Melhorias na Exibição
- Cabeçalhos em negrito e branco brilhante
- Campos vazios exibidos como "-" em cinza (mais limpo)
- Hostnames em branco brilhante quando presentes
- Separação visual clara entre tipos de dispositivos

### 🐛 Correções
- Removido código duplicado de atualização de vendor
- Corrigida lógica de atualização de MAC em dispositivos existentes
- Melhorada detecção de mudanças para salvar apenas quando necessário

---

## v0.1.0 - 2025-12-01

## Novas Funcionalidades Implementadas

### 🎯 1. Total de Dispositivos no Monitor
- **Descrição**: Exibe o total de endereços MAC detectados ao final da tabela
- **Localização**: `src/monitor.rs:253`
- **Benefício**: Visão rápida da quantidade de dispositivos na rede

---

### 🔄 2. Limpeza Automática do Cache ARP
- **Descrição**: Cache ARP é limpo antes de cada scan para garantir detecção de MACs atualizados
- **Localização**: `src/utils.rs:53-70` e `src/monitor.rs:83-86`
- **Requisitos**: Privilégios de root/sudo
- **Timeout do Scanner**: Aumentado de 3 para 10 segundos para melhor detecção
- **Delay entre requisições**: Aumentado de 500µs para 2ms para evitar flooding
- **Benefício**: Garante que os MACs detectados sejam os mais recentes

---

### 🌐 2.5. Resolução Aprimorada de Hostname (Multi-Método)
- **Descrição**: Sistema completo de resolução de hostname usando 4 métodos diferentes
- **Localização**: `src/utils.rs:77-243` e `src/scanner.rs:105-124`
- **Dependência**: `trust-dns-resolver = "0.23"`

#### Métodos de Resolução (em ordem de prioridade):

1. **DHCP Leases** (Mais rápido e confiável)
   - Lê arquivos de lease de DHCP de locais comuns
   - Suporta formatos: ISC DHCP, dnsmasq, Pi-hole
   - Locais verificados:
     - `/var/lib/dhcp/dhcpd.leases` (ISC DHCP)
     - `/tmp/dhcp.leases` (dnsmasq)
     - `/var/lib/misc/dnsmasq.leases` (dnsmasq Debian/Ubuntu)
     - `/etc/pihole/dhcp.leases` (Pi-hole)
   - Parse automático de ambos formatos (dnsmasq e ISC)

2. **DNS Reverso (PTR)**
   - Tentativa rápida de resolução tradicional
   - Filtra respostas que retornam apenas IPs

3. **mDNS (.local)**
   - Busca por padrões .local para dispositivos com Multicast DNS
   - Padrões testados: `ip-192-168-1-100.local`, `192-168-1-100.local`
   - Compatível com:
     - Dispositivos Apple (Bonjour/mDNS nativo)
     - Linux com Avahi
     - Alguns roteadores e smart devices

4. **NetBIOS Lookup**
   - Usa comando `nmblookup -A` se disponível
   - Detecta nomes de computadores Windows
   - Parse do formato NetBIOS para extrair hostname
   - Filtra nomes especiais e grupos

#### Recursos Adicionais:
- **Timeout Configurável**: 3 segundos por dispositivo (padrão)
- **Execução Assíncrona**: Não bloqueia o scan ARP
- **Fallback Inteligente**: Tenta próximo método se anterior falhar

#### Benefícios:
- ✅ **Cobertura Completa**: Windows, Linux, macOS, IoT
- ✅ **Performance**: DHCP leases são instantâneos
- ✅ **Confiabilidade**: 4 métodos de fallback
- ✅ **Timeout**: Não trava em dispositivos lentos

---

### 🎨 3. Vendor Lookup (OUI Database)
- **Descrição**: Identificação automática do fabricante baseado no endereço MAC
- **Localização**: `src/vendor.rs`
- **Banco de Dados**: 150+ vendors cadastrados
- **Funcionalidades**:
  - Identifica fabricantes de hardware (Intel, Realtek, Dell, HP, Apple, etc.)
  - Detecta máquinas virtuais (Proxmox, QEMU, VMware, VirtualBox, Hyper-V)
  - Identifica roteadores e switches (Cisco, Ubiquiti, TP-Link, Intelbras)
  - Detecta MACs privados/virtuais automaticamente
  - Destaque visual em **magenta** para VMs/containers

**Vendors de Virtualização Detectados**:
- `BC:24:11` → Proxmox Virtual Machine
- `52:54:00` → QEMU/KVM Virtual NIC
- `00:15:5D` → Microsoft Hyper-V
- `00:50:56`, `00:0C:29` → VMware
- `08:00:27` → Oracle VirtualBox
- `00:16:3E` → Xen Virtual Machine

---

### 🔧 4. Sistema de Mapeamento de MACs
- **Descrição**: Correção manual de MACs para ambientes virtualizados
- **Arquivo**: `mac_mapping.json`
- **Uso**: Para casos onde ARP detecta MAC da bridge/veth em vez do MAC real do container
- **Formato**:
  ```json
  [
    {
      "ip": "192.168.15.31",
      "real_mac": "BC:24:11:36:2D:6E",
      "description": "Proxmox LXC Container"
    }
  ]
  ```
- **Funcionalidades**:
  - Aplicado automaticamente ao carregar dispositivos
  - Salva MAC virtual para referência
  - Exibe ambos os MACs na tabela (real e virtual)

---

### 🛠️ 5. Módulo Proxmox API (Preparado)
- **Descrição**: Infraestrutura preparada para integração futura com API do Proxmox
- **Localização**: `src/proxmox.rs`
- **Status**: Implementado mas não ativo
- **Objetivo Futuro**: Obter automaticamente lista de VMs/containers e seus MACs reais

---

## 📊 Nova Interface do Monitor

### Antes:
```
IP              MAC               HOSTNAME             STATUS
```

### Agora:
```
IP              MAC               VENDOR                    VIRTUAL MAC          HOSTNAME             STATUS
------------------------------------------------------------------------------------------------------------------------
192.168.15.9    bc:24:11:36:2d:6e Proxmox Virtual Machine   -                    -                    Online
192.168.15.31   68:5b:35:8d:89:41 Intel                     -                    proxmox              Online
192.168.15.1    24:2f:d0:7f:b6:e0 Intelbras                 -                    _gateway             Online
------------------------------------------------------------------------------------------------------------------------
Total de dispositivos: 3
```

### Destaques Visuais:
- 🟣 **Vendors de VMs**: Exibidos em magenta
- 🔵 **MACs corrigidos**: Exibidos em cyan
- 🟢 **Devices Online**: Status em verde
- 🔴 **Devices Offline**: Status em vermelho

---

## 🐛 Problemas Conhecidos e Soluções

### Dispositivos que não respondem ARP
**Problema**: Alguns dispositivos (especialmente com firewall) não respondem a requisições ARP broadcast.

**Solução**:
1. Usar arquivo `mac_mapping.json` para mapeamento manual
2. Verificar firewall do dispositivo
3. Verificar se o dispositivo tem ARP habilitado

**Diagnóstico**: Execute o scan e verifique se o dispositivo responde:
```bash
sudo ./target/release/getmacrede scan -r 192.168.15.1-254
```

---

## 📁 Arquivos do Projeto

### Código Fonte:
- `src/main.rs` - Entrada principal e CLI
- `src/scanner.rs` - Scanner ARP (timeout: 5 segundos)
- `src/monitor.rs` - Monitor de rede contínuo
- `src/models.rs` - Estruturas de dados (Device, Status)
- `src/utils.rs` - Utilitários (parse IP, flush ARP)
- `src/vendor.rs` - Banco de dados OUI e lookup
- `src/proxmox.rs` - API Proxmox (futuro)

### Arquivos de Configuração:
- `devices.json` - Histórico de dispositivos detectados
- `blacklist.json` - Lista de MACs bloqueados
- `mac_mapping.json` - Mapeamento manual de MACs
- `mac_mapping.json.example` - Exemplo de mapeamento

### Documentação:
- `README_NOVAS_FUNCIONALIDADES.md` - Documentação das features
- `CHANGELOG.md` - Este arquivo

---

## 🚀 Como Usar

### Scan Único:
```bash
sudo ./target/release/getmacrede scan -r 192.168.15.1-254
```

### Monitor Contínuo:
```bash
sudo ./target/release/getmacrede monitor -r 192.168.15.1-254 -n 30
```

### Com Interface Específica:
```bash
sudo ./target/release/getmacrede monitor -r 192.168.15.1-254 -i enp1s0f0
```

---

## 🎉 Melhorias Implementadas

✅ Exibição do total de dispositivos
✅ Limpeza automática de cache ARP
✅ Timeout aumentado para 10 segundos
✅ Delay entre requisições ARP aumentado para 2ms
✅ **Resolução aprimorada de hostname com 4 métodos**:
   - DHCP leases (ISC DHCP, dnsmasq, Pi-hole)
   - DNS reverso (PTR)
   - mDNS (.local) para Apple/Linux
   - NetBIOS para Windows
✅ Timeout configurável de 3s por hostname
✅ Vendor lookup com 150+ fabricantes
✅ Detecção automática de VMs/containers
✅ Sistema de mapeamento de MACs
✅ Destaque visual para VMs (magenta)
✅ Coluna VENDOR na tabela
✅ Coluna VIRTUAL MAC para troubleshooting
✅ Estatísticas detalhadas (Total/Online/Offline/Blocked/VMs)
✅ Infraestrutura Proxmox API preparada

---

## 📝 Notas Técnicas

### Como Funciona o Vendor Lookup:
1. Extrai os primeiros 3 bytes do MAC (OUI - Organizationally Unique Identifier)
2. Busca o OUI em um HashMap interno (O(1))
3. Retorna o nome do fabricante
4. Se não encontrar, verifica se é MAC privado/virtual (bit U/L)

### Como Funciona o Mapeamento de MACs:
1. Carrega `mac_mapping.json` ao iniciar o monitor
2. Aplica mapeamento a todos os dispositivos (incluindo offline)
3. Salva MAC virtual no campo `virtual_mac`
4. Substitui MAC principal pelo real
5. Exibe ambos na tabela

### Como Funciona a Resolução de Hostname:

**Sistema Multi-Método com Fallback Inteligente:**

1. **DHCP Leases** (Método Primário - Mais confiável)
   - Função: `load_dhcp_leases()` em `utils.rs:79-143`
   - Lê múltiplos arquivos de lease DHCP conhecidos
   - Parse de 2 formatos principais:
     - **dnsmasq**: `timestamp mac ip hostname client-id`
     - **ISC DHCP**: `lease 192.168.1.100 { ... client-hostname "name"; }`
   - Retorna HashMap<IP, Hostname> para lookup O(1)
   - **Vantagem**: Instantâneo, não requer query de rede

2. **DNS Reverso (PTR)**
   - Usando `lookup_addr()` da biblioteca `dns-lookup`
   - Rápido mas falha em redes sem DNS reverso configurado
   - Filtra respostas que retornam apenas IPs

3. **mDNS (.local)** (Multicast DNS)
   - Cria `TokioAsyncResolver` para queries mDNS
   - Testa múltiplos padrões: `ip-192-168-1-100.local`, `192-168-1-100.local`
   - Verifica se a resolução retorna o IP correto antes de aceitar
   - **Compatível com**: macOS (Bonjour), Linux (Avahi), iOS

4. **NetBIOS Lookup** (Windows)
   - Função: `try_netbios_lookup()` em `utils.rs:145-173`
   - Executa `nmblookup -A <ip>` se comando disponível
   - Parse da saída NetBIOS: busca por `<00>` e `ACTIVE`
   - Filtra grupos e nomes especiais
   - **Detecta**: Nomes de computadores Windows na rede local

**Controle de Timeout:**
- Função wrapper: `resolve_hostname_with_timeout()` em `utils.rs:179-194`
- Usa `tokio::time::timeout()` para limite de 3 segundos
- Evita travamento em dispositivos não responsivos
- Se timeout expira, retorna `None` e continua

**Execução Assíncrona:**
- Scanner carrega DHCP leases uma vez: `utils::load_dhcp_leases()`
- Resolve hostnames após scan ARP completo (não bloqueia ARP)
- Processa múltiplos dispositivos em paralelo
- Cada dispositivo tem timeout individual de 3s

### Requisitos:
- Rust 1.91.1+
- Privilégios de root/sudo (para raw sockets e flush ARP)
- Linux com suporte a ARP
- Interface de rede ativa

### Requisitos Opcionais (para melhor detecção de hostname):
- **Para NetBIOS (Windows)**: Instalar `samba-common-bin` ou `samba`
  ```bash
  # Debian/Ubuntu
  sudo apt install samba-common-bin

  # Fedora/RHEL
  sudo dnf install samba-client

  # Arch Linux
  sudo pacman -S smbclient
  ```
- **Para mDNS (Apple/Linux)**: Avahi instalado (geralmente já vem por padrão)
- **Para DHCP Leases**: Acesso de leitura aos arquivos de lease do servidor DHCP

---

## 🔮 Melhorias Futuras Possíveis

- [ ] Integração completa com Proxmox API
- [ ] Download automático do banco OUI completo do IEEE
- [ ] Detecção de mudanças de MAC (MAC spoofing)
- [ ] Exportação de relatórios (CSV, JSON)
- [ ] Interface web
- [ ] Suporte para IPv6
- [ ] Detecção de duplicatas de MAC
- [ ] Alertas via webhook/email
- [ ] Integração com Nmap para port scanning

---

**Versão**: 0.1.0
**Data**: 2025-12-01
**Desenvolvido por**: Claude Code + neviim
