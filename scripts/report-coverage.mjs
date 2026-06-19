import fs from 'node:fs';
import path from 'node:path';

// 引数の取得
const [type, lcovPath] = process.argv.slice(2);

// 引数のバリデーション
if (!type || !lcovPath) {
  console.error('Usage: node report-coverage.mjs <rust|frontend> <lcov-file-path>');
  process.exit(1);
}

// ファイル存在確認
const resolvedPath = path.resolve(lcovPath);
if (!fs.existsSync(resolvedPath)) {
  console.error(`Error: LCOV file not found at ${resolvedPath}`);
  process.exit(1);
}

// カバレッジ情報の解析
const content = fs.readFileSync(resolvedPath, 'utf-8');
const lines = content.split(/\r?\n/);

let totalLF = 0;
let totalLH = 0;
let totalFNF = 0;
let totalFNH = 0;
let totalBRF = 0;
let totalBRH = 0;

// 行ごとの集計処理
for (const line of lines) {
  if (line.startsWith('LF:')) {
    totalLF += parseInt(line.substring(3).trim(), 10);
  } else if (line.startsWith('LH:')) {
    totalLH += parseInt(line.substring(3).trim(), 10);
  } else if (line.startsWith('FNF:')) {
    totalFNF += parseInt(line.substring(4).trim(), 10);
  } else if (line.startsWith('FNH:')) {
    totalFNH += parseInt(line.substring(4).trim(), 10);
  } else if (line.startsWith('BRF:')) {
    totalBRF += parseInt(line.substring(4).trim(), 10);
  } else if (line.startsWith('BRH:')) {
    totalBRH += parseInt(line.substring(4).trim(), 10);
  }
}

// パーセンテージ計算処理
const calculatePct = (hit, found) => {
  if (found === 0) return 100;
  return parseFloat(((hit / found) * 100).toFixed(2));
};

const linePct = calculatePct(totalLH, totalLF);
const fnPct = calculatePct(totalFNH, totalFNF);
const brPct = calculatePct(totalBRH, totalBRF);

// カバレッジステータス絵文字取得処理
const getStatusEmoji = (pct) => {
  if (pct >= 90) return '🟢';
  if (pct >= 50) return '🟡';
  return '🔴';
};

const lineEmoji = getStatusEmoji(linePct);
const fnEmoji = getStatusEmoji(fnPct);
const brEmoji = getStatusEmoji(brPct);

// レポート見出し用タイトル設定
const title = type === 'rust' ? '🦀 Rust Backend Coverage' : '⚡ JS Frontend Coverage';

// Markdownレポート生成処理
const markdownReport = `
### ${title}

| Metric | Coverage | Status | Details |
| :--- | :--- | :--- | :--- |
| **Lines** | ${linePct}% | ${lineEmoji} | ${totalLH} / ${totalLF} |
| **Functions** | ${fnPct}% | ${fnEmoji} | ${totalFNH} / ${totalFNF} |
| **Branches** | ${brPct}% | ${brEmoji} | ${totalBRH} / ${totalBRF} |
`;

// Job Summary への出力処理
const summaryFile = process.env.GITHUB_STEP_SUMMARY;
if (summaryFile) {
  try {
    fs.appendFileSync(summaryFile, markdownReport);
    console.log(`Coverage report appended to GITHUB_STEP_SUMMARY (${summaryFile})`);
  } catch (err) {
    console.warn(`Failed to write to GITHUB_STEP_SUMMARY: ${err.message}`);
  }
} else {
  console.log('--- Coverage Report Preview ---');
  console.log(markdownReport);
}

// PRコメント投稿および更新処理
async function postOrUpdatePRComment() {
  const token = process.env.GITHUB_TOKEN;
  const repo = process.env.GITHUB_REPOSITORY;
  const eventPath = process.env.GITHUB_EVENT_PATH;

  // 必要な環境変数の存在確認
  if (!token || !repo || !eventPath) {
    console.log('GitHub integration parameters missing. Skipping PR comment update.');
    return;
  }

  // イベントデータの読み込み
  let eventData;
  try {
    eventData = JSON.parse(fs.readFileSync(eventPath, 'utf8'));
  } catch (err) {
    console.warn(`Failed to read GITHUB_EVENT_PATH: ${err.message}`);
    return;
  }

  // PR番号の取得
  const prNumber = eventData.pull_request?.number;
  if (!prNumber) {
    console.log('Not a pull request context. Skipping PR comment update.');
    return;
  }

  // コメント識別用のユニークなHTMLコメントタグ
  const commentTag = `<!-- flmm-coverage-report-${type} -->`;
  const body = `${commentTag}\n${markdownReport}`;

  try {
    const listUrl = `https://api.github.com/repos/${repo}/issues/${prNumber}/comments`;
    const headers = {
      Authorization: `token ${token}`,
      Accept: 'application/vnd.github.v3+json',
      'User-Agent': 'flmm-coverage-reporter'
    };

    // 既存コメント一覧の取得
    const listRes = await fetch(listUrl, { headers });
    if (!listRes.ok) {
      throw new Error(`GET comments request failed with status: ${listRes.status}`);
    }

    const comments = await listRes.json();
    const existingComment = comments.find(c => c.body && c.body.includes(commentTag));

    if (existingComment) {
      // 既存コメントの更新処理
      const updateUrl = `https://api.github.com/repos/${repo}/issues/comments/${existingComment.id}`;
      const updateRes = await fetch(updateUrl, {
        method: 'PATCH',
        headers: {
          ...headers,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ body })
      });

      if (!updateRes.ok) {
        throw new Error(`PATCH comment request failed with status: ${updateRes.status}`);
      }
      console.log('PR coverage comment updated successfully.');
    } else {
      // 新規コメントの投稿処理
      const createRes = await fetch(listUrl, {
        method: 'POST',
        headers: {
          ...headers,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ body })
      });

      if (!createRes.ok) {
        throw new Error(`POST comment request failed with status: ${createRes.status}`);
      }
      console.log('PR coverage comment created successfully.');
    }
  } catch (err) {
    // 外部フォークからのPR等による権限エラーを想定したキャッチ処理
    // CIジョブを失敗させないための正常終了ハンドリング
    console.warn(`Could not post/update PR comment: ${err.message}`);
    console.log('Note: This is expected if the workflow is run from a fork repository or lacks permissions.');
  }
}

// 実行処理
await postOrUpdatePRComment();
