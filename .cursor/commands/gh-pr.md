# gh-pr

現在のブランチから GitHub PR を作成する。Conventional Commits に基づいてタイトルのプレフィックスを自動判定する。

## 手順

### 1. ブランチ確認

`git branch --show-current` で現在のブランチ名を取得する。master または main の場合はエラーとして中断し、フィーチャーブランチに切り替えるよう案内する。

### 2. コミット履歴の取得

`git log --oneline master..HEAD` を実行し、ベースブランチ (master) からの全コミットを一覧表示する。コミットが 0 件の場合は中断する。

### 3. 差分サマリの取得

`git diff master..HEAD --stat` を実行し、変更されたファイルの概要を確認する。

### 4. プレフィックスの自動判定

ブランチの内容（コミット履歴・メッセージ、差分サマリ）を総合的に解析し、以下の 3 種類から 1 つを自動判定する。複数に該当する場合は優先度が高いものを選択する:

1. **major bump** — 破壊的変更を含む場合 → `BREAKING CHANGE: title`
2. **minor bump** — 新機能の追加を含む場合 → `feat: title`
3. **patch bump** — バグ修正・軽微な変更のみの場合 → `fix: title`

`title` はブランチのコミット内容・差分から英語で 1 行に収まる要約を生成する。

### 5. PR タイトル・ボディの生成

- **タイトル**: `prefix: title` の形式で英語 1 行で生成する (例: `feat: Add version display at the bottom right of TUI`)
- **ボディ**: 以下のフォーマットで生成する:

```
## Summary
- 変更内容の箇条書き（日本語）

## Commits
- コミット一覧（git log --oneline の出力）
```

### 6. リモートへのプッシュと PR 作成

1. `git push -u origin HEAD` で現在のブランチをリモートにプッシュする
2. 以下の形式で `gh pr create` を実行する:

```bash
gh pr create --title "feat: Add version display at the bottom right of TUI" --body "$(cat <<'EOF'
## Summary
- ...

## Commits
- ...

EOF
)"
```

3. 作成された PR の URL をユーザーに報告する
