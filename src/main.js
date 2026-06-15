const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;
const { openUrl } = window.__TAURI__.opener;

// ─── Configurações Gerais ─────────────────────────────────────────────
const NEXUS_GAME_URL = 'https://www.nexusmods.com/007firstlight';
const DISCORD_URL = 'https://discord.gg/aMfk6wgA';

function getNexusModUrl() {
  const modId = localStorage.getItem('nexusModId') || '0';
  return modId !== '0' ? `https://www.nexusmods.com/007firstlight/mods/${modId}` : NEXUS_GAME_URL;
}

// ─── Dicionário de Traduções (i18n) ──────────────────────────────────
const translations = {
  en: {
    nav_home: "Home",
    nav_install: "Install Mod",
    nav_settings: "Settings",
    nav_about: "About",
    subtitle_home: "Mod Manager (.rpkg)",
    detecting: "Detecting...",
    scanning: "Scanning for installation...",
    checking: "Checking...",
    btn_install_main: "Install Mod",
    btn_uninstall: "Uninstall",
    btn_open_folder: "Open Folder",
    new_version: "New version available!",
    btn_download: "Download",
    lbl_game: "Game",
    lbl_mods: "Mods",
    lbl_backup: "Backup",
    lbl_platform: "Platform",
    val_not_installed: "Not installed",
    title_install: "Install Mod",
    subtitle_install: "Select the mod's .rpkg or .zip file",
    step1_title: "Game Detected",
    step1_not_found: "Not found",
    step1_btn: "Select manually",
    step2_title: "Mod File",
    step2_label: "Drag and drop <strong>.rpkg</strong> or <strong>.zip</strong> file here",
    step2_or: "or",
    step2_btn: "Browse file",
    step3_title: "Install",
    step3_warning: "A backup of the original files will be automatically created before installation.",
    btn_install_mod: "Install Mod",
    progress_label: "Installing...",
    title_settings: "Settings",
    subtitle_settings: "Customize the Mod Manager",
    settings_lang_title: "Language / Idioma",
    settings_lang_label: "Select Language / Escolha o Idioma",
    group_game_path: "Game Path",
    lbl_game_directory: "007: First Light Directory",
    btn_browse: "Browse",
    lbl_api_key: "API Key (optional — to check for updates)",
    placeholder_api_key: "Your Nexus Mods API key...",
    lbl_mod_id: "Nexus Mod ID (0 to disable)",
    placeholder_mod_id: "e.g., 123",
    lbl_auto_update: "Check for updates automatically",
    group_backup: "Backup",
    lbl_backup_status: "Original backup",
    btn_delete_backup: "Delete backup",
    btn_save_settings: "Save Settings",
    btn_check_updates_now: "Check for updates now",
    title_about: "About",
    subtitle_about: "Open Source Mod Manager",
    about_version: "Version 0.1.0",
    about_desc: "Open-source mod manager for <strong>007: First Light</strong>. Safely and easily install, update, and remove .rpkg files without needing technical knowledge.",
    abt_discord_title: "Discord Community",
    abt_discord_sub: "Visit the official server",
    abt_tech_title: "Technologies used",
    about_footer: "Open Source · 2026 · MIT / Apache 2.0 License",
    
    // Dynamic keys in JS
    active_mods: "Active Mod(s)",
    no_active_mods: "No active mods",
    no_mods: "No mods",
    active: "Active",
    available: "Available",
    backup_created: "Backup created",
    no_backup: "No backup found",
    game_installed: "✅ Installed",
    game_not_found: "❌ Not found",
    scanning_not_found: "Not found — select manually",
    toast_settings_saved: "Settings saved!",
    toast_game_dir_configured: "Game directory configured!",
    confirm_uninstall: "Do you want to uninstall all mods and restore original game files?",
    progress_backup: "Creating backup of original files...",
    progress_copy: "Copying mod files...",
    progress_patch: "Patching packagedefinition.txt...",
    progress_done: "Completed!",
    err_detecting: "Error detecting game: ",
    err_select_file: "Error selecting file: ",
    err_select_dir: "Error selecting directory: ",
    err_install: "Error installing mod: ",
    err_uninstall: "Error uninstalling: ",
    err_check_updates: "Could not check for updates: ",
    updates_disabled: "Updates check disabled (Mod ID not configured).",
    toast_no_backup: "No backup to remove.",
    toast_dev_feature: "Feature in development.",
    drop_error_ext: "Please select a .rpkg or .zip file",
    update_banner_ver: "Version {version} available on Nexus Mods.",
    toast_latest_ver: "You already have the latest version!",
    game_not_found_manual: "Game not found. Configure path manually.",
    title_installed_mods: "Installed Mods",
    no_mods_installed: "No mods installed yet. Go to \"Install Mod\" to add one!",
    confirm_delete_mod: "Are you sure you want to permanently delete this mod?",
    mod_status_active: "Active",
    mod_status_inactive: "Inactive",
    author_by: "by",
    manual_label: "Manual",
    preview_title: "Package contents",
    preview_summary: "Install plan",
    preview_no_rpkg: "No valid .rpkg files found in this package.",
    preview_target: "Target",
    preview_original: "Source",
    preview_warnings: "Warnings",
    preview_metadata: "Metadata detected",
    preview_no_metadata: "No metadata",
    preview_inspecting: "Inspecting package...",
    preview_inspect_failed: "Could not inspect package: ",
    toast_backup_deleted: "Backup deleted.",
    steam_label: "🟦 Steam",
    epic_label: "🟪 Epic Games",
    unknown_label: "❓ Unknown"
  },
  pt: {
    nav_home: "Início",
    nav_install: "Instalar Mod",
    nav_settings: "Configurações",
    nav_about: "Sobre",
    subtitle_home: "Gerenciador de Mods (.rpkg)",
    detecting: "Detectando...",
    scanning: "Procurando instalação...",
    checking: "Verificando...",
    btn_install_main: "Instalar Mod",
    btn_uninstall: "Desinstalar",
    btn_open_folder: "Abrir Pasta",
    new_version: "Nova versão disponível!",
    btn_download: "Baixar",
    lbl_game: "Jogo",
    lbl_mods: "Mods",
    lbl_backup: "Backup",
    lbl_platform: "Plataforma",
    val_not_installed: "Não instalado",
    title_install: "Instalar Mod",
    subtitle_install: "Selecione o arquivo .rpkg ou .zip do mod",
    step1_title: "Jogo Detectado",
    step1_not_found: "Não encontrado",
    step1_btn: "Selecionar manualmente",
    step2_title: "Arquivo do Mod",
    step2_label: "Arraste o arquivo <strong>.rpkg</strong> ou <strong>.zip</strong> aqui",
    step2_or: "ou",
    step2_btn: "Procurar arquivo",
    step3_title: "Instalar",
    step3_warning: "Um backup dos arquivos originais será criado automaticamente antes da instalação.",
    btn_install_mod: "Instalar Mod",
    progress_label: "Instalando...",
    title_settings: "Configurações",
    subtitle_settings: "Personalize o Mod Manager",
    settings_lang_title: "Language / Idioma",
    settings_lang_label: "Select Language / Escolha o Idioma",
    group_game_path: "Caminho do Jogo",
    lbl_game_directory: "Diretório do 007: First Light",
    btn_browse: "Alterar",
    lbl_api_key: "API Key (opcional — para verificar atualizações)",
    placeholder_api_key: "Sua API key do Nexus Mods...",
    lbl_mod_id: "Nexus Mod ID (0 para desativar)",
    placeholder_mod_id: "ex: 123",
    lbl_auto_update: "Verificar atualizações automaticamente",
    group_backup: "Backup",
    lbl_backup_status: "Backup original",
    btn_delete_backup: "Remover backup",
    btn_save_settings: "Salvar configurações",
    btn_check_updates_now: "Verificar atualizações agora",
    title_about: "Sobre",
    subtitle_about: "Gerenciador de Mods Open Source",
    about_version: "Versão 0.1.0",
    about_desc: "Gerenciador de mods de código aberto para o <strong>007: First Light</strong>. Instale, atualize e remova arquivos .rpkg de forma simples e segura, sem precisar de conhecimento técnico.",
    abt_discord_title: "Comunidade Discord",
    abt_discord_sub: "Visite o servidor oficial",
    abt_tech_title: "Tecnologias utilizadas",
    about_footer: "Open Source · 2026 · Licença MIT / Apache 2.0",
    
    // Dynamic keys in JS
    active_mods: "Mod(s) ativo(s)",
    no_active_mods: "Nenhum mod ativo",
    no_mods: "Nenhum mod",
    active: "Ativo",
    available: "Disponível",
    backup_created: "Backup original criado",
    no_backup: "Nenhum backup",
    game_installed: "✅ Instalado",
    game_not_found: "❌ Não encontrado",
    scanning_not_found: "Não encontrado — selecione manualmente",
    toast_settings_saved: "Configurações salvas!",
    toast_game_dir_configured: "Pasta do jogo configurada!",
    confirm_uninstall: "Deseja desinstalar todos os mods e restaurar os arquivos originais do jogo?",
    progress_backup: "Criando backup dos arquivos originais...",
    progress_copy: "Copiando arquivos do mod...",
    progress_patch: "Atualizando packagedefinition.txt...",
    progress_done: "Concluído!",
    err_detecting: "Erro ao detectar o jogo: ",
    err_select_file: "Erro ao selecionar arquivo: ",
    err_select_dir: "Erro ao selecionar pasta: ",
    err_install: "Erro ao instalar: ",
    err_uninstall: "Erro ao desinstalar: ",
    err_check_updates: "Não foi possível verificar atualizações: ",
    updates_disabled: "Verificação de atualizações desativada (Mod ID não configurado).",
    toast_no_backup: "Nenhum backup para remover.",
    toast_dev_feature: "Funcionalidade em desenvolvimento.",
    drop_error_ext: "Por favor, selecione um arquivo .rpkg ou .zip",
    update_banner_ver: "Versão {version} disponível no Nexus Mods.",
    toast_latest_ver: "Você já tem a versão mais recente!",
    game_not_found_manual: "Jogo não encontrado. Configure o caminho manualmente.",
    title_installed_mods: "Mods Instalados",
    no_mods_installed: "Nenhum mod instalado ainda. Vá em \"Instalar Mod\" para adicionar um!",
    confirm_delete_mod: "Tem certeza que deseja excluir permanentemente este mod?",
    mod_status_active: "Ativo",
    mod_status_inactive: "Inativo",
    author_by: "por",
    manual_label: "Manual",
    preview_title: "Conteúdo do pacote",
    preview_summary: "Plano de instalação",
    preview_no_rpkg: "Nenhum arquivo .rpkg válido encontrado neste pacote.",
    preview_target: "Destino",
    preview_original: "Origem",
    preview_warnings: "Avisos",
    preview_metadata: "Metadados detectados",
    preview_no_metadata: "Sem metadados",
    preview_inspecting: "Inspecionando pacote...",
    preview_inspect_failed: "Não foi possível inspecionar o pacote: ",
    toast_backup_deleted: "Backup removido.",
    steam_label: "🟦 Steam",
    epic_label: "🟪 Epic Games",
    unknown_label: "❓ Desconhecido"
  }
};

// ─── Estado global ────────────────────────────────────────────────────
let state = {
  gamePath: '',
  platform: '',
  gameFound: false,
  modInstalled: false,
  modVersion: '',
  backupExists: false,
  selectedModFile: '',
  modPreview: null,
  settings: {
    game_path: '',
    language: 'en',
    nexus_api_key: '',
    nexus_mod_id: '0',
    auto_check_updates: true
  },
  language: 'en' // Default language
};

// ─── Utilitários de Tradução ──────────────────────────────────────────
function applyLanguage(lang) {
  if (lang !== 'en' && lang !== 'pt') {
    lang = 'en';
  }
  state.language = lang;
  document.querySelectorAll('[data-i18n]').forEach(el => {
    const key = el.getAttribute('data-i18n');
    if (translations[lang] && translations[lang][key]) {
      if (translations[lang][key].includes('<strong') || translations[lang][key].includes('<span') || translations[lang][key].includes('<strong>')) {
        el.innerHTML = translations[lang][key];
      } else {
        el.textContent = translations[lang][key];
      }
    }
  });

  document.querySelectorAll('[data-i18n-placeholder]').forEach(el => {
    const key = el.getAttribute('data-i18n-placeholder');
    if (translations[lang] && translations[lang][key]) {
      el.setAttribute('placeholder', translations[lang][key]);
    }
  });

  const select = document.getElementById('select-language');
  if (select) {
    select.value = lang;
  }
  renderModPreview(state.modPreview);
  renderModList().catch(() => {});
}

// ─── Utilitários ──────────────────────────────────────────────────────
function toast(msg, type = 'info') {
  const container = document.getElementById('toast-container');
  const el = document.createElement('div');
  el.className = `toast toast-${type}`;

  const icons = { success: '✅', error: '❌', info: 'ℹ️' };
  el.innerHTML = `
    <span class="toast-icon">${icons[type] || 'ℹ️'}</span>
    <span class="toast-msg">${msg}</span>
  `;
  container.appendChild(el);

  setTimeout(() => {
    el.classList.add('fadeout');
    el.addEventListener('animationend', () => el.remove());
  }, 4000);
}

function setProgress(label, percent) {
  const area = document.getElementById('progress-area');
  const bar  = document.getElementById('progress-bar');
  const lbl  = document.getElementById('progress-label');
  area.style.display = 'block';
  lbl.textContent = label;
  bar.style.width = percent + '%';
}

function hideProgress() {
  document.getElementById('progress-area').style.display = 'none';
  document.getElementById('progress-bar').style.width = '0%';
}

function normalizeSettings(settings = {}) {
  return {
    game_path: settings.game_path || settings.gamePath || localStorage.getItem('gamePath') || '',
    language: settings.language === 'pt' ? 'pt' : 'en',
    nexus_api_key: settings.nexus_api_key || settings.nexusApiKey || localStorage.getItem('nexusApiKey') || '',
    nexus_mod_id: settings.nexus_mod_id || settings.nexusModId || localStorage.getItem('nexusModId') || '0',
    auto_check_updates: typeof settings.auto_check_updates === 'boolean'
      ? settings.auto_check_updates
      : localStorage.getItem('autoCheckUpdates') !== 'false'
  };
}

function currentSettingsFromInputs() {
  return normalizeSettings({
    ...state.settings,
    game_path: document.getElementById('input-game-path').value.trim(),
    language: document.getElementById('select-language').value,
    nexus_api_key: document.getElementById('input-nexus-key').value.trim(),
    nexus_mod_id: document.getElementById('input-mod-id').value.trim() || '0',
    auto_check_updates: document.getElementById('check-auto-update').checked
  });
}

function applySettings(settings) {
  const normalized = normalizeSettings(settings);
  state.settings = normalized;
  state.language = normalized.language;

  document.getElementById('input-game-path').value = normalized.game_path;
  document.getElementById('input-nexus-key').value = normalized.nexus_api_key;
  document.getElementById('input-mod-id').value = normalized.nexus_mod_id;
  document.getElementById('check-auto-update').checked = normalized.auto_check_updates;
  document.getElementById('select-language').value = normalized.language;

  localStorage.setItem('language', normalized.language);
  localStorage.setItem('gamePath', normalized.game_path);
  localStorage.setItem('nexusApiKey', normalized.nexus_api_key);
  localStorage.setItem('nexusModId', normalized.nexus_mod_id);
  localStorage.setItem('autoCheckUpdates', normalized.auto_check_updates.toString());
}

async function persistSettings(overrides = {}) {
  const next = normalizeSettings({ ...currentSettingsFromInputs(), ...overrides });
  const saved = await invoke('save_settings', { settings: next });
  applySettings(saved);
  return state.settings;
}

function platformLabel(platform) {
  const lang = state.language;
  const labels = {
    steam: translations[lang].steam_label,
    epic: translations[lang].epic_label,
    manual: translations[lang].manual_label,
    unknown: translations[lang].unknown_label
  };
  return labels[platform] || '—';
}

function applyGameInfo(info) {
  state.gamePath = info.path || '';
  state.platform = info.platform || 'unknown';
  state.gameFound = Boolean(info.found);

  const lang = state.language;
  const gamePathDisplay = document.getElementById('game-path-display');
  gamePathDisplay.textContent = state.gameFound ? state.gamePath : translations[lang].game_not_found_manual;
  gamePathDisplay.style.color = state.gameFound ? '' : 'var(--danger)';

  document.getElementById('platform-tag').textContent = platformLabel(state.platform);
  document.getElementById('sc-platform').textContent = state.gameFound ? platformLabel(state.platform).replace(/^[^\w]+ /, '') : '—';
  document.getElementById('sc-game').textContent = state.gameFound ? translations[lang].game_installed : translations[lang].game_not_found;
  document.getElementById('step-game-path-text').textContent = state.gameFound ? state.gamePath : translations[lang].scanning_not_found;
  document.getElementById('input-game-path').value = state.gamePath;
  document.getElementById('btn-open-folder').disabled = !state.gameFound;
  updateInstallButtonState();
}

function escapeHtml(value) {
  return String(value ?? '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

function formatBytes(bytes) {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value.toFixed(value >= 10 || unit === 0 ? 0 : 1)} ${units[unit]}`;
}

function renderModPreview(preview) {
  const box = document.getElementById('mod-preview');
  const type = document.getElementById('mod-preview-type');
  const summary = document.getElementById('mod-preview-summary');
  const list = document.getElementById('mod-preview-list');
  const warnings = document.getElementById('mod-preview-warnings');
  if (!box || !type || !summary || !list || !warnings) return;

  if (!preview) {
    box.style.display = 'none';
    type.textContent = '';
    summary.textContent = '';
    list.innerHTML = '';
    warnings.innerHTML = '';
    return;
  }

  const lang = state.language;
  const rpkgFiles = preview.rpkg_files || [];
  box.style.display = 'block';
  type.textContent = (preview.package_type || '').toUpperCase();
  summary.textContent = `${translations[lang].preview_summary}: ${rpkgFiles.length} .rpkg · ${preview.has_metadata ? translations[lang].preview_metadata : translations[lang].preview_no_metadata}`;

  list.innerHTML = rpkgFiles.length
    ? rpkgFiles.map(item => `
        <div class="mod-preview-row">
          <div class="mod-preview-file">
            <span class="mod-preview-label">${translations[lang].preview_target}</span>
            <span class="mod-preview-name">${escapeHtml(item.target_name)}</span>
          </div>
          <div class="mod-preview-meta">
            <span>${translations[lang].preview_original}: ${escapeHtml(item.original_name)}</span>
            <span>chunk${item.chunk} patch${item.target_patch}</span>
            <span>${formatBytes(item.size)}</span>
          </div>
        </div>
      `).join('')
    : `<div class="mod-preview-empty">${translations[lang].preview_no_rpkg}</div>`;

  warnings.innerHTML = (preview.warnings || []).length
    ? `
      <div class="mod-preview-warning-title">${translations[lang].preview_warnings}</div>
      ${(preview.warnings || []).map(item => `<div class="mod-preview-warning">${escapeHtml(item)}</div>`).join('')}
    `
    : '';
}

async function inspectSelectedMod() {
  if (!state.selectedModFile) {
    state.modPreview = null;
    renderModPreview(null);
    return;
  }

  const lang = state.language;
  state.modPreview = {
    file_name: state.selectedModFile.split(/[\\/]/).pop(),
    package_type: state.selectedModFile.split('.').pop() || '',
    installable: false,
    rpkg_files: [],
    has_packagedefinition: false,
    has_metadata: false,
    warnings: [translations[lang].preview_inspecting]
  };
  renderModPreview(state.modPreview);
  updateInstallButtonState();

  try {
    state.modPreview = await invoke('inspect_mod', {
      modPath: state.selectedModFile,
      gamePath: state.gamePath || null
    });
  } catch (err) {
    state.modPreview = {
      ...state.modPreview,
      warnings: [translations[lang].preview_inspect_failed + err]
    };
    toast(translations[lang].preview_inspect_failed + err, 'error');
  }

  renderModPreview(state.modPreview);
  updateInstallButtonState();
}

// ─── Navegação por abas ───────────────────────────────────────────────
document.querySelectorAll('.nav-item').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('.nav-item').forEach(b => b.classList.remove('active'));
    document.querySelectorAll('.tab-panel').forEach(p => p.classList.remove('active'));
    btn.classList.add('active');
    document.getElementById('tab-' + btn.dataset.tab).classList.add('active');
  });
});

// ─── Detectar jogo ────────────────────────────────────────────────────
async function detectGame() {
  try {
    const info = await invoke('detect_game');
    applyGameInfo(info);

    if (info.found) {
      state.settings.game_path = info.path;
      await getModStatus();
    } else {
      setModStatusUI(false, '', false);
    }
    await renderModList();
    if (state.selectedModFile) {
      await inspectSelectedMod();
    }
  } catch (err) {
    console.error('detect_game error:', err);
    toast(translations[state.language].err_detecting + err, 'error');
  }
}

// ─── Status do mod ────────────────────────────────────────────────────
async function getModStatus() {
  if (!state.gamePath) return;
  try {
    const status = await invoke('get_mod_status', { gamePath: state.gamePath });
    state.modInstalled  = status.installed;
    state.modVersion    = status.version;
    state.backupExists  = status.backup_exists;
    setModStatusUI(status.installed, status.version, status.backup_exists);
  } catch (err) {
    console.error('get_mod_status error:', err);
  }
}

function setModStatusUI(installed, version, backupExists) {
  const chip    = document.getElementById('mod-status-chip');
  const verChip = document.getElementById('version-chip');
  const verText = document.getElementById('installed-version-text');
  const scMod   = document.getElementById('sc-mod');
  const scBack  = document.getElementById('sc-backup');
  const backupStatusText = document.getElementById('backup-status-text');
  const lang = state.language;

  if (installed) {
    chip.className = 'status-chip installed';
    chip.querySelector('.chip-text').textContent = translations[lang].active_mods;
    verChip.style.display = 'flex';
    verText.textContent = translations[lang].active;
    scMod.textContent = translations[lang].active;
  } else {
    chip.className = 'status-chip not-installed';
    chip.querySelector('.chip-text').textContent = translations[lang].no_active_mods;
    verChip.style.display = 'none';
    scMod.textContent = translations[lang].no_mods;
  }

  scBack.textContent = backupExists ? '✅ ' + translations[lang].available : '—';
  backupStatusText.textContent = backupExists ? '✅ ' + translations[lang].backup_created : '❌ ' + translations[lang].no_backup;

  // Botões de ação
  const btnInstallMain   = document.getElementById('btn-install-main');
  const btnUninstallMain = document.getElementById('btn-uninstall-main');

  btnInstallMain.disabled   = !state.gameFound;
  btnUninstallMain.disabled = !backupExists;
  updateInstallButtonState();
}

// ─── Instalar mod ─────────────────────────────────────────────────────
async function installMod() {
  if (!state.gamePath || !state.selectedModFile) return;

  const btn = document.getElementById('btn-do-install');
  btn.disabled = true;
  const lang = state.language;

  setProgress(translations[lang].progress_backup, 20);
  await sleep(400);
  setProgress(translations[lang].progress_copy, 55);
  await sleep(300);
  setProgress(translations[lang].progress_patch, 80);

  try {
    const result = await invoke('install_mod', {
      gamePath: state.gamePath,
      modPath: state.selectedModFile,
      lang: state.language
    });
    setProgress(translations[lang].progress_done, 100);
    await sleep(600);
    hideProgress();
    toast(result, 'success');
    await getModStatus();
    await renderModList();
    // Volta para home
    document.getElementById('nav-home').click();
  } catch (err) {
    hideProgress();
    toast(translations[state.language].err_install + err, 'error');
    btn.disabled = false;
  }
}

// ─── Desinstalar mod ──────────────────────────────────────────────────
async function uninstallMod() {
  if (!state.gamePath) return;

  const lang = state.language;
  const confirmed = confirm(translations[lang].confirm_uninstall);
  if (!confirmed) return;

  const btn = document.getElementById('btn-uninstall-main');
  btn.disabled = true;

  try {
    const result = await invoke('uninstall_mod', { gamePath: state.gamePath, lang: state.language });
    toast(result, 'success');
    await getModStatus();
    await renderModList();
  } catch (err) {
    toast(translations[state.language].err_uninstall + err, 'error');
    btn.disabled = false;
  }
}

// ─── Verificar atualizações ───────────────────────────────────────────
async function checkUpdates(showToast = false) {
  const lang = state.language;
  const settings = normalizeSettings({ ...state.settings, ...currentSettingsFromInputs() });
  const modId = settings.nexus_mod_id || '0';
  const apiKey = settings.nexus_api_key || '';

  if (modId === '0' || !modId) {
    if (showToast) toast(translations[lang].updates_disabled, 'info');
    return;
  }
  try {
    const result = await invoke('check_updates', { 
      currentVersion: state.modVersion || '0.1.0', 
      modId: modId,
      apiKey: apiKey || null
    });
    if (result.has_update) {
      const banner = document.getElementById('update-banner');
      banner.style.display = 'flex';
      document.getElementById('update-version-text').textContent = translations[lang].update_banner_ver.replace('{version}', result.version);
    } else {
      document.getElementById('update-banner').style.display = 'none';
      if (showToast) {
        toast(translations[lang].toast_latest_ver, 'success');
      }
    }
  } catch (err) {
    if (showToast) toast(translations[state.language].err_check_updates + err, 'error');
  }
}

// ─── Selecionar arquivo do mod ────────────────────────────────────────
async function browseModFile() {
  try {
    const lang = state.language;
    const selected = await open({
      multiple: false,
      filters: [
        { name: translations[lang].step2_title, extensions: ['rpkg', 'zip'] },
      ],
    });
    if (selected) {
      await setModFile(selected);
    }
  } catch (err) {
    toast(translations[state.language].err_select_file + err, 'error');
  }
}

async function setModFile(filePath) {
  state.selectedModFile = filePath;
  const name = filePath.split(/[\\/]/).pop();

  document.getElementById('drop-zone').style.display = 'none';
  const display = document.getElementById('selected-file-display');
  display.style.display = 'flex';
  document.getElementById('selected-file-name').textContent = name;
  await inspectSelectedMod();
  updateInstallButtonState();
}

function clearModFile() {
  state.selectedModFile = '';
  state.modPreview = null;
  document.getElementById('drop-zone').style.display = 'block';
  document.getElementById('selected-file-display').style.display = 'none';
  renderModPreview(null);
  updateInstallButtonState();
}

function updateInstallButtonState() {
  const btn = document.getElementById('btn-do-install');
  btn.disabled = !state.gamePath || !state.selectedModFile || !state.modPreview?.installable;
}

// ─── Selecionar pasta do jogo manualmente ────────────────────────────
async function selectGamePath() {
  try {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      const saved = await persistSettings({ game_path: selected });
      applyGameInfo({ found: true, path: saved.game_path, platform: 'manual' });
      await getModStatus();
      await renderModList();
      if (state.selectedModFile) {
        await inspectSelectedMod();
      }
      toast(translations[state.language].toast_game_dir_configured, 'success');
    }
  } catch (err) {
    toast(translations[state.language].err_select_dir + err, 'error');
  }
}

// ─── Abrir pasta do jogo ─────────────────────────────────────────────
async function openGameFolder() {
  try {
    await invoke('open_game_folder', { gamePath: state.gamePath });
  } catch (err) {
    toast(translations[state.language].err_select_dir + err, 'error');
  }
}

// ─── Links externos ───────────────────────────────────────────────────
async function openLink(url) {
  try {
    await openUrl(url);
  } catch {
    window.open(url, '_blank');
  }
}

// ─── Drag & drop ──────────────────────────────────────────────────────
function setupDropZone() {
  const zone = document.getElementById('drop-zone');
  zone.addEventListener('dragover', e => { e.preventDefault(); zone.classList.add('drag-over'); });
  zone.addEventListener('dragleave', () => zone.classList.remove('drag-over'));
  zone.addEventListener('drop', async e => {
    e.preventDefault(); zone.classList.remove('drag-over');
    const file = e.dataTransfer.files[0];
    if (file && (file.name.endsWith('.rpkg') || file.name.endsWith('.zip'))) {
      await setModFile(file.path || file.name);
    } else {
      toast(translations[state.language].drop_error_ext, 'error');
    }
  });
}

// ─── Salvar configurações ─────────────────────────────────────────────
async function saveSettings() {
  try {
    const saved = await persistSettings();
    if (saved.game_path) {
      applyGameInfo({ found: true, path: saved.game_path, platform: 'manual' });
    }

    toast(translations[state.language].toast_settings_saved, 'success');
    await getModStatus();
    await renderModList();
    if (state.selectedModFile) {
      await inspectSelectedMod();
    }
    if (saved.auto_check_updates) {
      checkUpdates(false).catch(() => {});
    } else {
      document.getElementById('update-banner').style.display = 'none';
    }
  } catch (err) {
    toast(translations[state.language].err_select_dir + err, 'error');
  }
}

// ─── Renderizar Lista de Mods Instalados ─────────────────────────────────
async function renderModList() {
  const listSection = document.getElementById('mod-list-section');
  const listGrid    = document.getElementById('mod-list-grid');
  if (!listSection || !listGrid) return;

  if (!state.gameFound || !state.gamePath) {
    listSection.style.display = 'none';
    return;
  }

  try {
    const mods = await invoke('list_mods', { gamePath: state.gamePath });
    listSection.style.display = 'block';

    if (mods.length === 0) {
      listGrid.innerHTML = `
        <div class="no-mods-placeholder">
          ${translations[state.language].no_mods_installed}
        </div>
      `;
      return;
    }

    listGrid.innerHTML = '';
    mods.forEach(m => {
      const card = document.createElement('div');
      card.className = 'mod-card';

      const titleText = escapeHtml(m.name);
      const verBadge  = m.version ? `<span class="mod-card-version">v${escapeHtml(m.version)}</span>` : '';
      const authorText = m.author ? `<span class="mod-card-author">${translations[state.language].author_by} ${escapeHtml(m.author)}</span>` : '';
      const descText = escapeHtml(m.description || m.filename);
      const fileText = m.original_filename && m.original_filename !== m.filename
        ? `${escapeHtml(m.original_filename)} → ${escapeHtml(m.filename)}`
        : escapeHtml(m.filename);
      const patchText = `chunk${m.chunk} patch${m.patch}`;

      card.innerHTML = `
        <div class="mod-card-info">
          <div class="mod-card-title-row">
            <span class="mod-card-title">${titleText}</span>
            ${verBadge}
            ${authorText}
          </div>
          <div class="mod-card-desc">${descText}</div>
          <div class="mod-card-file">${fileText} · ${patchText}</div>
        </div>
        <div class="mod-card-controls">
          <label class="toggle">
            <input type="checkbox" class="toggle-mod-active" data-mod-id="${m.id}" ${m.active ? 'checked' : ''} />
            <span class="toggle-slider"></span>
          </label>
          <button class="btn-delete-mod" data-mod-id="${m.id}" title="${translations[state.language].btn_uninstall}">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/></svg>
          </button>
        </div>
      `;

      // Bind toggle change
      card.querySelector('.toggle-mod-active').addEventListener('change', async (e) => {
        const modId = e.target.getAttribute('data-mod-id');
        const active = e.target.checked;
        try {
          await invoke('toggle_mod', { gamePath: state.gamePath, modId, active });
          await getModStatus();
          await renderModList();
        } catch (err) {
          toast(err, 'error');
          e.target.checked = !active;
        }
      });

      // Bind delete click
      card.querySelector('.btn-delete-mod').addEventListener('click', async () => {
        const modId = m.id;
        const confirmed = confirm(translations[state.language].confirm_delete_mod);
        if (!confirmed) return;
        try {
          await invoke('delete_mod', { gamePath: state.gamePath, modId });
          await getModStatus();
          await renderModList();
        } catch (err) {
          toast(err, 'error');
        }
      });

      listGrid.appendChild(card);
    });
  } catch (err) {
    console.error('list_mods error:', err);
    listSection.style.display = 'none';
  }
}

// ─── Helpers ─────────────────────────────────────────────────────────
function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

// ─── Event listeners ──────────────────────────────────────────────────
function bindEvents() {
  // Home
  document.getElementById('btn-install-main').addEventListener('click', () => {
    document.getElementById('nav-install').click();
  });
  document.getElementById('btn-uninstall-main').addEventListener('click', uninstallMod);
  document.getElementById('btn-open-folder').addEventListener('click', openGameFolder);

  // Install tab
  document.getElementById('btn-browse-mod').addEventListener('click', (e) => {
    e.stopPropagation();
    browseModFile();
  });
  document.getElementById('drop-zone').addEventListener('click', browseModFile);
  document.getElementById('btn-clear-file').addEventListener('click', clearModFile);
  document.getElementById('btn-do-install').addEventListener('click', installMod);
  document.getElementById('btn-manual-path').addEventListener('click', selectGamePath);

  // Settings
  document.getElementById('btn-choose-path').addEventListener('click', selectGamePath);
  document.getElementById('btn-save-settings').addEventListener('click', saveSettings);
  document.getElementById('btn-check-now').addEventListener('click', () => checkUpdates(true));
  document.getElementById('btn-delete-backup').addEventListener('click', async () => {
    if (!state.backupExists) { toast(translations[state.language].toast_no_backup, 'info'); return; }
    try {
      await invoke('delete_backup', { gamePath: state.gamePath });
      toast(translations[state.language].toast_backup_deleted, 'success');
      await getModStatus();
    } catch (err) {
      toast(err, 'error');
    }
  });

  // Language Selection Listener
  const selectLang = document.getElementById('select-language');
  if (selectLang) {
    selectLang.addEventListener('change', (e) => {
      const selectedLang = e.target.value;
      state.settings.language = selectedLang;
      persistSettings({ language: selectedLang }).catch(err => toast(err, 'error'));
      applyLanguage(selectedLang);
      getModStatus();
      detectGame();
    });
  }

  // Update banner
  document.getElementById('btn-download-update').addEventListener('click', e => {
    e.preventDefault();
    openLink(getNexusModUrl());
  });

  // About links
  document.getElementById('abt-discord').addEventListener('click', e => { e.preventDefault(); openLink(DISCORD_URL); });
  document.getElementById('abt-nexus').addEventListener('click',   e => { e.preventDefault(); openLink(NEXUS_GAME_URL); });
  document.getElementById('link-discord').addEventListener('click', e => { e.preventDefault(); openLink(DISCORD_URL); });
  document.getElementById('link-nexus').addEventListener('click',   e => { e.preventDefault(); openLink(NEXUS_GAME_URL); });
}

// ─── Init ─────────────────────────────────────────────────────────────
async function init() {
  bindEvents();
  setupDropZone();

  try {
    const loaded = await invoke('load_settings');
    applySettings(loaded);
  } catch (err) {
    console.warn('load_settings error:', err);
    applySettings(normalizeSettings());
  }

  applyLanguage(state.settings.language);
  if (state.settings.game_path) {
    try {
      await persistSettings({ game_path: state.settings.game_path });
    } catch (err) {
      console.warn('settings migration skipped:', err);
    }
  }
  await detectGame();

  if (state.settings.auto_check_updates) {
    checkUpdates(false).catch(() => {});
  }
}

window.addEventListener('DOMContentLoaded', init);

// ユニットテスト/E2Eテスト用の純粋関数のグローバル公開
window.formatBytes = formatBytes;
window.escapeHtml = escapeHtml;
window.normalizeSettings = normalizeSettings;

