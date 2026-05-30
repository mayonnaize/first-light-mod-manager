# 🕵️ First Light Mod Manager (FLMM)

> A clean, modern, and lightweight open-source mod manager for **007: First Light**.

![License](https://img.shields.io/github/license/Welsker-dev/first-light-mod-manager)
![Platform](https://img.shields.io/badge/platform-Windows-blue)
![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri-24C8D8)
![Status](https://img.shields.io/badge/status-in%20development-yellow)

---

## 🌎 English

**First Light Mod Manager (FLMM)** is a free, open-source mod manager built specifically for **007: First Light** (IO Interactive, 2026), powered by the **Glacier 2 Engine**.

Inspired by the modding tools developed for the **Hitman: World of Assassination** community (also on Glacier 2), FLMM brings the same convenience to 007: First Light — making it easy to install, manage, and remove `.rpkg` and `.zip` mods without touching any files manually.

### ✨ Features

- 🔍 **Auto-Detection** — Automatically finds your game install directory (Steam & Epic Games Store)
- 📦 **One-Click Install** — Drag and drop `.rpkg` or `.zip` mod packages to install
- ✍️ **Smart Manifest Patching** — Automatically edits `packagedefinition.txt` to inject mods cleanly
- 🛡️ **Safe Backups** — Backs up your original `Runtime` folder before applying any mod
- 🗑️ **One-Click Uninstall** — Fully restore your game to vanilla state instantly
- 📋 **Activity Log** — Real-time log of every operation
- ⚡ **Lightweight** — Under 10MB, no runtime dependencies required

### 🚀 How to Use

1. **Download** the installer or portable `.exe` from [Releases](../../releases)
2. **Open FLMM** — it will auto-detect your 007: First Light install
3. If not detected, click **"Select manually"** and point to your game folder
4. Go to the **"Install Mod"** tab
5. **Drag and drop** your `.rpkg` or `.zip` mod file (or click to browse)
6. Click **"Install Mod"** — done!
7. To remove all mods, click **"Uninstall"** on the home screen

### 🛠️ Requirements

- Windows 10 / 11 (64-bit)
- 007: First Light installed via Steam or Epic Games Store

---

## 🇧🇷 Português

**First Light Mod Manager (FLMM)** é um gerenciador de mods gratuito e open-source feito especificamente para **007: First Light** (IO Interactive, 2026), que roda na **Glacier 2 Engine**.

Inspirado pelas ferramentas de modding da comunidade de **Hitman: World of Assassination** (também na Glacier 2), o FLMM traz a mesma praticidade para o 007: First Light — facilitando instalar, gerenciar e remover mods `.rpkg` e `.zip` sem precisar mexer em nenhum arquivo manualmente.

### ✨ Recursos

- 🔍 **Detecção Automática** — Encontra a pasta do jogo sozinho (Steam e Epic Games)
- 📦 **Instalação Simplificada** — Arraste e solte arquivos `.rpkg` ou `.zip` diretamente na tela
- ✍️ **Modificação Segura** — Atualiza o `packagedefinition.txt` injetando os mods de forma limpa
- 🛡️ **Backup Original** — Cria backup da pasta `Runtime` antes de instalar qualquer mod
- 🗑️ **Restauração Rápida** — Remove todos os mods e restaura o jogo original com 1 clique
- 📋 **Log em Tempo Real** — Acompanhe cada etapa da instalação
- ⚡ **Leve** — Menos de 10MB, sem dependências extras

### 🚀 Como Usar

1. **Baixe** o instalador ou `.exe` portátil em [Releases](../../releases)
2. **Abra o FLMM** — ele detecta automaticamente a pasta do 007: First Light
3. Se não encontrar, clique em **"Selecionar manualmente"** e aponte para a pasta do jogo
4. Vá na aba **"Instalar Mod"**
5. **Arraste e solte** seu arquivo `.rpkg` ou `.zip` (ou clique para procurar)
6. Clique em **"Instalar Mod"** — pronto!
7. Para remover todos os mods, clique em **"Desinstalar"** na tela inicial

### 🛠️ Requisitos

- Windows 10 / 11 (64-bit)
- 007: First Light instalado via Steam ou Epic Games Store

---

## 🏗️ Built With

- [Tauri](https://tauri.app/) — Lightweight desktop app framework (Rust + HTML/CSS/JS)
- [Rust](https://www.rust-lang.org/) — Backend logic (file operations, game detection, RPKG handling)
- Vanilla HTML/CSS/JS — Frontend interface

## 🔗 Related Projects & Credits

- [Glacier Modding](https://glaciermodding.org/) — Community tools for Glacier 2 Engine games
- [RPKG Tool](https://github.com/glacier-modding/RPKG-Tool) — The tool that inspired the RPKG handling in FLMM
- [Simple Mod Framework](https://www.nexusmods.com/hitman3/mods/200) — Mod manager for Hitman: WoA, which inspired this project

## 🤝 Contributing

Pull requests are welcome! If you find a bug or want to suggest a feature, open an [Issue](../../issues).

## 📄 License

This project is licensed under the [MIT License](LICENSE).

---

> Made with ❤️ by [DublaX Team](https://discord.gg/aMfk6wgA) — *"Dublamos jogos!"*
>
> 007 FIRST LIGHT © 2026 IOI / Metro-Goldwyn-Mayer Studios Inc. This tool is not affiliated with IO Interactive, Danjaq, or EON Productions.
