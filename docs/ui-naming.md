# UI名称ガイド

## 1. 目的と対象読者

このドキュメントは、UI改善時に「どの領域の話か」を明確にするための命名ルールです。  
デザイン仕様そのものではなく、用語を統一するための参照資料です。

対象読者:
- UI改善Issueを書く人
- 実装する人
- レビューする人

## 2. 画面座標と語彙ルール

画面の座標は次の4領域で表現します。
- `Top`: 画面上段
- `Middle`: 画面中段
- `Bottom`: 画面下段
- `Popup`: オーバーレイ表示

語彙ルール:
- `Bar`: 1行入力や短い情報表示の横長領域
- `Pane`: 複数行の主表示領域
- `Panel`: 補助情報を表示する領域
- `Popup`: モーダル的に重なる領域

## 3. 共通領域の正式名称

- `Current Directory Bar`
  - 位置: `Top`
  - 画面タイトル: `current directory`
  - 役割: 現在ディレクトリ表示
- `File List Pane`
  - 位置: `Middle`
  - 画面タイトル: `files (x/y)`
  - 役割: エントリ一覧表示
- `Shell Output Popup`
  - 位置: `Popup`
  - 画面タイトル: `shell output: ... (exit ...)`
  - 役割: shell実行結果表示

`Status Bar` は `Browse` モード時のみ `Bottom` に表示されます（モード別領域を参照）。

## 4. モード別領域名称

### Browse
- `Status Bar`
  - 位置: `Bottom`
  - 画面タイトル: `status`
  - 内容:
    - 折りたたみ（初期）: ステータスメッセージまたは `Modified`, `Size` の1行表示
    - 展開（`m`）: `Modified`, `Size`, `Perm`, `Owner`, `Group` を表示
    - 狭幅時: 見切れ防止のため1項目1行へ自動切替

### Filter
- `Filter Input Bar`
  - 位置: `Bottom`
  - 画面タイトル: `filter (/): Enter apply Esc clear`
  - 内容: `/<query>`

### Command
- `Command Candidates Pane`
  - 位置: `Bottom`（上段）
  - 画面タイトル: `commands (Tab next / Shift+Tab prev)`
  - 内容: コマンド候補一覧
- `Command Input Bar`
  - 位置: `Bottom`（下段）
  - 画面タイトル: `command (:): Enter run Esc cancel Tab select Shift+Tab reverse`
  - 内容: `:<query>`（カーソルは末尾表示）

### Shell
- `Shell Input Bar`
  - 位置: `Bottom`
  - 画面タイトル: `shell (!): Enter run Esc cancel`
  - 内容: `!<command>`
- `Shell Help Panel`
  - 位置: `Bottom`（Shell入力バーの下）
  - 画面タイトル: `shell`
  - 内容: 実行ヒント（`Enter: run shell command` / `Esc: cancel`）

### Create
- `Create Insert Row`
  - 位置: `Middle`（File List Pane 内、選択中エントリの直下）
  - 画面タイトル: なし（リスト行として表示）
  - 内容: `Icon <input>` + 右寄せで `// \`/\`[Folder name] or [File name]`（`/` プレフィックスでフォルダアイコン、それ以外でファイルアイコン）
- `Create Help Panel`
  - 位置: `Bottom`
  - 画面タイトル: `create (n)`
  - 内容: `Enter: create  Esc: cancel`

## 5. 用語対応表

| 正式名称 | 画面表示タイトル | コード上の描画箇所 | 表示条件 (Mode) |
|---|---|---|---|
| Current Directory Bar | `current directory` | `draw(): path_block / path_para` | 全Mode |
| File List Pane | `files (x/y)` | `draw(): list_block / list` | 全Mode |
| Status Bar | `status` | `draw(): block / para` (Browse分岐) | `Mode::Browse` |
| Filter Input Bar | `filter (/): Enter apply Esc clear` | `draw(): filter_block / filter_para` | `Mode::Filter` |
| Command Candidates Pane | `commands (Tab next / Shift+Tab prev)` | `draw(): cand_block / cand_list` | `Mode::Command` |
| Command Input Bar | `command (:): Enter run Esc cancel Tab select Shift+Tab reverse` | `draw(): cmd_block / cmd_para` | `Mode::Command` |
| Shell Input Bar | `shell (!): Enter run Esc cancel` | `draw(): shell_block / shell_para` | `Mode::Shell` |
| Shell Help Panel | `shell` | `draw(): panel_block / panel_para` | `Mode::Shell` |
| Shell Output Popup | `shell output: ... (exit ...)` | `draw(): popup_area / popup para` | `app.show_shell_popup == true` |
| Create Insert Row | （リスト行） | `draw(): list items`（選択行直下に挿入） | `Mode::Create` |
| Create Help Panel | `create (n)` | `draw(): block / para` (Create分岐) | `Mode::Create` |

## 6. UI改善Issue/PR テンプレート

Issue/PRには次の書式を使ってください。

```md
## UI変更対象
- 対象領域: <正式名称>
- 変更内容: レイアウト / 配色 / キー挙動 / 文言
- 影響モード: Browse / Filter / Command / Shell / Create

## 受け入れ条件
- 対象領域がこの命名ガイドで一意に参照できること
- モード別表示条件に矛盾がないこと
```

## 7. 変更ルール

- `src/ui.rs` のタイトル文字列を変更した場合は、必ずこのドキュメントを更新する。
- 新しい表示領域を追加した場合は、次を必ず追加する。
  - 正式名称
  - モード条件
  - 用語対応表の行
- 既存名称を変更する場合は、Issue/PR本文で旧名称と新名称を併記する。
