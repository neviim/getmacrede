# Novas Funcionalidades Implementadas

## 📦 Versão 0.2.0 - Sistema de Cores e Correção de Duplicados

### 🎨 Sistema de Cores Intuitivo

O monitor agora utiliza um esquema de cores semântico que permite identificação visual instantânea dos dispositivos:

#### Paleta de Cores Implementada:

```
🔵 IP em Cyan        → Máquina Virtual / Container
⚪ IP em Branco      → Dispositivo Físico

🟢 MAC em Verde      → Hardware físico real (saudável)
🔵 MAC em Cyan Bold  → MAC real de VM (quando mapeado)
⚫ MAC vazio (-)     → Sem MAC real conhecido

🟡 VIRTUAL MAC em Amarelo → Interface virtual detectada

🟢 Status Online    → Dispositivo ativo na rede
🔴 Status Offline   → Dispositivo inativo
🔴 Status Block     → Bloqueado (bold)

🟣 Vendor em Magenta → Tecnologia de virtualização
⚪ Vendor em Branco  → Fabricante físico conhecido
⚫ Vendor vazio (-)  → Fabricante desconhecido
```

#### Legenda Visual Automática

Uma legenda é exibida automaticamente no topo da tabela:

```
Legend: IP: □ Physical | □ VM/Virtual | MAC: □ Physical | □ VM Real | □ VM Virtual
```

**Benefícios:**
- ✅ Identificação instantânea de VMs vs dispositivos físicos
- ✅ Detecção visual rápida de problemas (MACs desconhecidos, dispositivos offline)
- ✅ Cores consistentes e intuitivas
- ✅ Acessibilidade melhorada

**Localização**: `src/monitor.rs:281-390`

---

### 🔧 Correção Automática de IPs Duplicados

#### Problema Resolvido

Em ambientes com VMs/containers, MACs virtuais podem mudar frequentemente, causando:
- Múltiplas entradas para o mesmo IP
- Poluição visual na tabela
- Dificuldade de rastreamento

**Exemplo do Problema:**
```
192.168.15.144  8e:9c:33:ee:d0:a6  Offline  Virtual/Private MAC
192.168.15.144  5e:fc:17:1f:61:92  Offline  Virtual/Private MAC
192.168.15.144  5c:01:3b:81:08:80  Offline
```

#### Solução Implementada

O sistema agora:
1. **Rastreia por IP** ao invés de MAC (chave primária = IP)
2. **Remove duplicados automaticamente** ao carregar dados
3. **Mantém apenas a entrada mais recente** para cada IP
4. **Atualiza MACs dinamicamente** quando detecta mudanças

**Depois da Correção:**
```
192.168.15.144                    5c:01:3b:81:08:80  Offline
```

**Benefícios:**
- ✅ Cada IP aparece apenas uma vez
- ✅ Migração automática de dados antigos
- ✅ Rastreamento correto de dispositivos com MAC dinâmico
- ✅ Tabela limpa e organizada

**Localização**: `src/monitor.rs:56-90, 119-192`

---

### 🎯 Detecção Automática de Virtual MAC

#### Funcionalidade

O sistema agora detecta **automaticamente** quando um MAC é virtual e o move para a coluna apropriada.

#### Métodos de Detecção:

1. **OUI Database** - Verifica se o OUI (primeiros 3 bytes) corresponde a um fabricante de virtualização
2. **Bit U/L** - Detecta "locally administered addresses" (bit 2 do primeiro byte)
3. **Mapeamento Manual** - Integra com `mac_mapping.json` quando disponível

#### Lógica de Exibição:

| Situação | Coluna MAC | Coluna VIRTUAL MAC |
|----------|------------|-------------------|
| MAC virtual SEM mapeamento | `-` (vazio) | MAC virtual (amarelo) |
| MAC virtual COM mapeamento | MAC real (cyan bold) | MAC virtual (amarelo) |
| MAC físico | MAC físico (verde) | `-` (vazio) |

#### Exemplo Visual:

```
IP              MAC               VIRTUAL MAC       VENDOR
----------------------------------------------------------------
192.168.15.5    -                 ca:4e:2b:74:e3:fa Unknown (Private/Virtual)
192.168.15.6    64:1c:67:5f:34:da bc:24:11:0e:b2:cb Proxmox Virtual Machine
192.168.15.10   d0:94:66:a8:d8:72 -                 Intel
```

**Benefícios:**
- ✅ Virtual MACs sempre na coluna correta
- ✅ Distinção clara entre MACs reais e virtuais
- ✅ Facilita identificação de VMs sem mapeamento manual
- ✅ Migração automática ao carregar dados existentes

**Localização**: `src/monitor.rs:68-72, 112-118, 330-341` | `src/vendor.rs:176-191`

---

## 📦 Versão 0.1.0 - Funcionalidades Base

## 1. Total de Dispositivos no Monitor

O monitor agora exibe o total de endereços MAC listados ao final da tabela.

**Localização**: `src/monitor.rs:222`

```
Total de dispositivos: X
```

---

## 2. Limpeza Automática do Cache ARP

Antes de cada scan, o cache ARP é automaticamente limpo para garantir detecção de MACs atualizados.

**Localização**: `src/utils.rs:53-70` e `src/monitor.rs:62-65`

**Requisitos**: Requer privilégios de root/sudo

**Comando**: `sudo ./target/release/getmacrede monitor -r 192.168.15.1-254`

---

## 3. Correção de MAC Addresses para Ambientes Virtualizados

### Problema Resolvido

Em ambientes de virtualização (Proxmox, VMware, etc.), o ARP pode retornar o MAC da interface virtual (bridge/veth) em vez do MAC real da VM/container.

### Solução Implementada

Sistema de mapeamento manual de IP -> MAC real através do arquivo `mac_mapping.json`.

### Como Usar

1. **Criar arquivo de mapeamento** (baseado no exemplo):
   ```bash
   cp mac_mapping.json.example mac_mapping.json
   ```

2. **Editar o arquivo** `mac_mapping.json`:
   ```json
   [
     {
       "ip": "192.168.15.31",
       "real_mac": "BC:24:11:36:2D:6E",
       "description": "Proxmox LXC Container"
     }
   ]
   ```

3. **Executar o monitor** normalmente - o mapeamento será aplicado automaticamente

### Exibição no Monitor

A tabela agora mostra:
- **MAC**: O endereço MAC real (em cyan se foi corrigido)
- **VIRTUAL MAC**: O MAC virtual detectado pelo ARP (se houver correção)

```
IP              MAC               VIRTUAL MAC       HOSTNAME             STATUS
--------------------------------------------------------------------------------------
192.168.15.31   BC:24:11:36:2D:6E 68:5b:35:8d:89:41 container-name       Online
```

---

## 4. Módulo de Integração com Proxmox API (Preparado para Futuro)

**Localização**: `src/proxmox.rs`

Módulo preparado para integração futura com a API do Proxmox para obtenção automática de MACs reais.

**Status**: Código implementado mas não ativo (aguardando configuração completa)

---

## Arquivos Criados/Modificados

### Novos Arquivos
- `src/proxmox.rs` - Módulo para API Proxmox e mapeamento de MACs
- `mac_mapping.json.example` - Arquivo de exemplo para mapeamento manual

### Arquivos Modificados
- `src/monitor.rs` - Adicionado total, limpeza ARP e correção de MACs
- `src/utils.rs` - Função para limpar cache ARP
- `src/models.rs` - Campo `virtual_mac` adicionado ao Device
- `Cargo.toml` - Dependência `reqwest` adicionada

---

## Por Que o Scanner Detectava o MAC Errado?

### Causa do Problema

No Proxmox (e outras plataformas de virtualização), quando você tem:
- IP: `192.168.15.31`
- MAC Real do Container: `BC:24:11:36:2D:6E`
- MAC Detectado: `68:5b:35:8d:89:41`

O que acontece:

1. **Proxy ARP**: O host Proxmox responde às requisições ARP em nome do container
2. **Interface Virtual (veth)**: O MAC `68:5b:35:8d:89:41` é da interface virtual que faz bridge
3. **Camada de Rede**: O tráfego passa pelo host antes de chegar ao container

### Como a Solução Funciona

1. Scanner detecta o MAC virtual via ARP (normal)
2. Sistema verifica se existe mapeamento para aquele IP
3. Se existir, substitui o MAC pelo real e armazena o virtual
4. Exibe ambos os MACs no monitor para transparência

---

## Execução

### Modo Scan Normal
```bash
sudo ./target/release/getmacrede scan -r 192.168.15.1-254
```

### Modo Monitor com Mapeamento de MACs
```bash
# 1. Configure o mapeamento
cp mac_mapping.json.example mac_mapping.json
nano mac_mapping.json

# 2. Execute com sudo (necessário para limpar cache ARP)
sudo ./target/release/getmacrede monitor -r 192.168.15.1-254 -n 30
```

---

## Benefícios

- ✅ Total de dispositivos exibido claramente
- ✅ Cache ARP sempre limpo = detecção mais precisa
- ✅ MACs reais de VMs/containers corretamente identificados
- ✅ Rastreamento de MAC virtual para troubleshooting
- ✅ Pronto para integração futura com APIs de virtualização
