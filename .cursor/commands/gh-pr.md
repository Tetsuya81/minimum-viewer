# gh-pr

現在のブランチから GitHub PR を作成する。Conventional Commits に基づいてタイトルのプレフィックスを自動判定する。

## 手順

以下の手順を順に実行すること。

### 1. ブランチ確認

`git branch --show-current` で現在のブランチ名を取得する。master または main の場合はエラーとして中断し、フィーチャーブランチに切り替えるよう案内する。

### 2. コミット履歴の取得

`git log --oneline master..HEAD` を実行し、ベースブランチ (master) からの全コミットを一覧取得する。コミットが 0 件の場合は中断する。

### 3. 差分サマリの取得

`git diff master..HEAD --stat` を実行し、変更されたファイルの概要を取得する。

### 4. プレフィックスの自動判定

ステップ 2・3 で取得したコミット履歴と差分サマリを総合的に分析し、以下の 3 種類から 1 つを判定する。複数に該当する場合は優先度が高いものを選択する:

1. **BREAKING CHANGE** — 破壊的変更を含む場合（API 変更、既存機能の削除など）
2. **feat** — 新機能の追加を含む場合
3. **fix** — バグ修正・軽微な変更のみの場合（デフォルト）

### 5. PR タイトル・ボディの生成

コミット内容と差分を分析し、以下を生成する:

- **タイトル**: `prefix: English summary` の形式。コミット内容・差分から英語で 1 行に収まる要約を生成する。
  - 例: `feat: Add version display at the bottom right of TUI`
  - ブランチ名ではなく、実際の変更内容から生成すること
- **ボディ**: 以下のフォーマットで生成する:

```
## Summary
- 変更内容の箇条書き（日本語、複数項目可）

## Commits
(git log --oneline master..HEAD の出力をそのまま貼り付け)
```

Summary セクションはコミット群と diff を読み、変更点を日本語の箇条書きで列挙する。

### 6. リモートへのプッシュと PR 作成

1. `git push -u origin HEAD` で現在のブランチをリモートにプッシュする
2. 以下の形式で `gh pr create` を実行する:

```bash
gh pr create --title "prefix: English summary" --body "$(cat <<'EOF'
## Summary
- ...

## Commits
- ...

EOF
)"
```

3. 作成された PR の URL をユーザーに報告する
