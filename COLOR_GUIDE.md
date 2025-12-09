# 🎨 Guia de Cores - GetMacRede

## Visão Geral

O GetMacRede v0.2.0+ utiliza um sistema de cores **semântico e intuitivo** que permite identificação visual instantânea dos dispositivos na rede.

---

## 📊 Paleta de Cores Completa

### IP (Coluna 1)

| Cor | Código ANSI | Significado | Quando Aparece |
|-----|-------------|-------------|----------------|
| **Branco** | Normal | Dispositivo físico | MAC não virtual, vendor não indica VM |
| **Cyan** | `\033[36m` | VM/Container | Virtual MAC presente OU vendor indica virtualização |

**Exemplo:**
```
192.168.15.10    ← Branco (dispositivo físico - Intel)
192.168.15.6     ← Cyan (VM - Proxmox Virtual Machine)
```

---

### MAC (Coluna 2)

| Cor | Código ANSI | Significado | Quando Aparece |
|-----|-------------|-------------|----------------|
| **Verde** | `\033[32m` | Hardware físico | MAC físico real, sem virtualização |
| **Cyan Bold** | `\033[1;36m` | MAC real de VM | MAC físico de VM (quando mapeado) |
| **- (cinza)** | `\033[90m` | Desconhecido | Apenas virtual MAC conhecido |

**Exemplo:**
```
d0:94:66:a8:d8:72  ← Verde (placa Intel física)
bc:24:11:0e:b2:cb  ← Cyan Bold (MAC real de VM Proxmox)
-                  ← Cinza (sem MAC real conhecido)
```

---

### VIRTUAL MAC (Coluna 3)

| Cor | Código ANSI | Significado | Quando Aparece |
|-----|-------------|-------------|----------------|
| **Amarelo** | `\033[33m` | Interface virtual | MAC virtual detectado |
| **(vazio)** | - | Sem virtual | Dispositivo físico comum |

**Exemplo:**
```
ca:4e:2b:74:e3:fa  ← Amarelo (interface virtual bridge/veth)
                   ← Vazio (dispositivo físico, sem virtualização)
```

---

### HOSTNAME (Coluna 4)

| Cor | Código ANSI | Significado | Quando Aparece |
|-----|-------------|-------------|----------------|
| **Branco Brilhante** | `\033[97m` | Nome configurado | Hostname resolvido com sucesso |
| **- (cinza)** | `\033[90m` | Sem nome | Hostname não disponível |

**Exemplo:**
```
gateway            ← Branco brilhante (hostname resolvido)
-                  ← Cinza (sem hostname)
```

---

### STATUS (Coluna 5)

| Cor | Código ANSI | Significado | Quando Aparece |
|-----|-------------|-------------|----------------|
| **Verde** | `\033[32m` | Online | Respondeu ao último scan |
| **Vermelho** | `\033[31m` | Offline | Não respondeu por 2.5x o intervalo |
| **Vermelho Bold** | `\033[1;31m` | Bloqueado | MAC está em `blacklist.json` |

**Exemplo:**
```
Online   ← Verde (dispositivo ativo)
Offline  ← Vermelho (dispositivo inativo)
Block    ← Vermelho Bold (bloqueado)
```

---

### VENDOR (Coluna 6)

| Cor | Código ANSI | Significado | Quando Aparece |
|-----|-------------|-------------|----------------|
| **Magenta** | `\033[35m` | Virtualização | Proxmox, QEMU, VMware, Hyper-V, VirtualBox |
| **Branco** | `\033[37m` | Fabricante físico | Intel, Realtek, Apple, Samsung, etc. |
| **- (cinza)** | `\033[90m` | Desconhecido | OUI não encontrado no database |

**Exemplo:**
```
Proxmox Virtual Machine  ← Magenta (tecnologia de virtualização)
Intel                    ← Branco (fabricante conhecido)
-                        ← Cinza (OUI desconhecido)
```

---

## 🎯 Lógica de Detecção

### Como o Sistema Determina as Cores

```rust
// 1. Detecta se é virtual
is_virtual = device.virtual_mac.is_some() ||
             vendor.contains("Virtual|Proxmox|QEMU|VMware|...")

// 2. Define cor do IP
ip_color = if is_virtual { CYAN } else { WHITE }

// 3. Define cor do MAC
if virtual_mac.is_some() && mac == virtual_mac {
    mac_color = EMPTY (cinza)
} else if virtual_mac.is_some() {
    mac_color = CYAN_BOLD  // MAC real de VM
} else {
    mac_color = GREEN      // MAC físico
}

// 4. Virtual MAC sempre amarelo (se presente)
virtual_mac_color = YELLOW

// 5. Vendor
vendor_color = if is_virtual { MAGENTA }
               else if vendor.is_empty() { GRAY }
               else { WHITE }
```

---

## 📖 Exemplos Práticos

### 1. Dispositivo Físico Comum

```
192.168.15.10   d0:94:66:a8:d8:72                   -        Online     Intel
```

| Campo | Cor | Por quê |
|-------|-----|---------|
| IP | Branco | Não é virtual |
| MAC | Verde | Hardware físico |
| VIRTUAL MAC | Vazio | Sem virtualização |
| Status | Verde | Online |
| Vendor | Branco | Fabricante conhecido |

---

### 2. VM Proxmox com Mapeamento

```
192.168.15.6    bc:24:11:0e:b2:cb ca:4e:2b:74:e3:fa -        Online     Proxmox Virtual Machine
```

| Campo | Cor | Por quê |
|-------|-----|---------|
| IP | Cyan | É virtual (vendor = Proxmox) |
| MAC | Cyan Bold | MAC real de VM (mapeado) |
| VIRTUAL MAC | Amarelo | Interface virtual detectada |
| Status | Verde | Online |
| Vendor | Magenta | Tecnologia de virtualização |

---

### 3. VM sem Mapeamento Real

```
192.168.15.103  -                 ba:ff:f6:99:50:60 -        Offline    Virtual/Private MAC
```

| Campo | Cor | Por quê |
|-------|-----|---------|
| IP | Cyan | É virtual (virtual_mac presente) |
| MAC | Cinza (-) | MAC real desconhecido |
| VIRTUAL MAC | Amarelo | Interface virtual |
| Status | Vermelho | Offline |
| Vendor | Magenta | Virtual/Private MAC |

---

### 4. Dispositivo Bloqueado

```
192.168.15.99   aa:bb:cc:dd:ee:ff -                 -        Block      Unknown
```

| Campo | Cor | Por quê |
|-------|-----|---------|
| IP | Branco | Não é virtual |
| MAC | Verde | Hardware físico |
| VIRTUAL MAC | Vazio | Sem virtualização |
| Status | **Vermelho Bold** | Em blacklist.json |
| Vendor | Cinza | Desconhecido |

---

## 💡 Dicas de Interpretação

### Identificação Rápida

| Você Vê | Significa |
|---------|-----------|
| 🔵 IP Cyan | Foco em VMs/Containers |
| 🟢 MAC Verde | Hardware físico confiável |
| 🟡 VIRTUAL MAC Amarelo | Atenção: interface virtualizada |
| 🟣 Vendor Magenta | Tecnologia de virtualização |
| 🔴 Status Vermelho Bold | **ATENÇÃO**: Dispositivo bloqueado |

### Combinações Comuns

#### ✅ Dispositivo Físico Saudável
```
Branco | Verde | Vazio | Verde | Branco
```

#### 🔵 VM com Mapeamento Completo
```
Cyan | Cyan Bold | Amarelo | Verde | Magenta
```

#### ⚠️ VM sem MAC Real Conhecido
```
Cyan | Cinza (-) | Amarelo | Vermelho | Magenta
```

#### 🚫 Dispositivo Bloqueado
```
Branco | Verde | Vazio | Vermelho Bold | Qualquer
```

---

## 🛠️ Personalização

As cores estão definidas em `src/monitor.rs` (linhas 297-390). Para personalizar:

```rust
// Exemplo: Mudar cor de VMs de Cyan para Blue
let ip_colored = if is_virtual {
    device.ip.blue().to_string()  // Altere aqui
} else {
    device.ip.to_string()
};
```

### Cores Disponíveis (crate `colored`)

- `.black()`, `.red()`, `.green()`, `.yellow()`
- `.blue()`, `.magenta()`, `.cyan()`, `.white()`
- `.bright_black()`, `.bright_red()`, etc.
- `.bold()`, `.dimmed()`, `.italic()`, `.underline()`

---

## 📚 Referências

- **Código**: `src/monitor.rs:297-390`
- **Vendor DB**: `src/vendor.rs:10-193`
- **CHANGELOG**: `CHANGELOG.md` (v0.2.0)
- **Documentação**: `README_NOVAS_FUNCIONALIDADES.md`

---

**Versão**: 0.2.0
**Última Atualização**: 2025-12-03
