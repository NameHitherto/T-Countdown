/**
 * 版本发布脚本
 * 用法: node scripts/bump-version.js <版本号>
 * 示例: node scripts/bump-version.js 0.4.0
 *
 * 执行流程:
 * 1. 更新 package.json、src-tauri/tauri.conf.json、src-tauri/Cargo.toml 的版本号
 * 2. 读取 RELEASE_NOTES.md 内容，写入 .github/workflows/release.yml 的 releaseBody
 * 3. 自动 git commit 并在最新提交上打 tag，然后推送 tag
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const newVersion = process.argv[2];
if (!newVersion || !/^\d+\.\d+\.\d+$/.test(newVersion)) {
  console.error('用法: node scripts/bump-version.js <版本号>');
  console.error('示例: node scripts/bump-version.js 0.4.0');
  process.exit(1);
}

const root = path.resolve(__dirname, '..');
const tag = `v${newVersion}`;

// ========== 1. 更新版本号 ==========

// package.json
const pkgPath = path.join(root, 'package.json');
const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf-8'));
const oldVersion = pkg.version;
pkg.version = newVersion;
fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n');
console.log(`package.json: ${oldVersion} -> ${newVersion}`);

// tauri.conf.json
const tauriPath = path.join(root, 'src-tauri', 'tauri.conf.json');
const tauri = JSON.parse(fs.readFileSync(tauriPath, 'utf-8'));
tauri.version = newVersion;
fs.writeFileSync(tauriPath, JSON.stringify(tauri, null, 2) + '\n');
console.log(`tauri.conf.json: ${oldVersion} -> ${newVersion}`);

// Cargo.toml
const cargoPath = path.join(root, 'src-tauri', 'Cargo.toml');
let cargo = fs.readFileSync(cargoPath, 'utf-8');
cargo = cargo.replace(
  /^(version\s*=\s*")[\d]+\.[\d]+\.[\d]+[^"]*(")/m,
  `$1${newVersion}$2`
);
fs.writeFileSync(cargoPath, cargo);
console.log(`Cargo.toml: ${oldVersion} -> ${newVersion}`);

// ========== 2. 将 RELEASE_NOTES.md 写入 release.yml ==========

const notesPath = path.join(root, 'RELEASE_NOTES.md');
if (!fs.existsSync(notesPath)) {
  console.error('未找到 RELEASE_NOTES.md，请先编写发布说明');
  process.exit(1);
}

const notes = fs.readFileSync(notesPath, 'utf-8').trimEnd();
// 将 release notes 内容缩进 12 个空格（YAML block scalar 格式）
const indentedNotes = notes
  .split('\n')
  .map((line) => '            ' + line)
  .join('\n');

const ymlPath = path.join(root, '.github', 'workflows', 'release.yml');
let yml = fs.readFileSync(ymlPath, 'utf-8');

// 替换 releaseBody 块：匹配 "releaseBody: |" 及其后续缩进行
yml = yml.replace(
  /releaseBody: \|[\s\S]*?(?=\n          \w)/,
  `releaseBody: |\n${indentedNotes}\n`
);

fs.writeFileSync(ymlPath, yml);
console.log(`release.yml: releaseBody 已更新`);

// ========== 3. Git commit + tag + push ==========

try {
  execSync('git add -A', { cwd: root, stdio: 'inherit' });
  execSync(`git commit -m "release: ${tag}"`, { cwd: root, stdio: 'inherit' });
  console.log(`已提交: release: ${tag}`);

  execSync(`git tag ${tag}`, { cwd: root, stdio: 'inherit' });
  console.log(`已创建 tag: ${tag}`);

  execSync(`git push`, { cwd: root, stdio: 'inherit' });
  execSync(`git push origin ${tag}`, { cwd: root, stdio: 'inherit' });
  console.log(`已推送 tag: ${tag}`);

  console.log(`\n发布完成！GitHub Actions 将自动构建 ${tag}`);
} catch (e) {
  console.error('Git 操作失败:', e.message);
  process.exit(1);
}
