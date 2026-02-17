# minimum-viewer: Command と Key bindings 統合リスト

現状の `COMMAND_SPECS`（`src/command/mod.rs`）と各モードのキー処理（`src/main.rs`）を、  
**「コマンドを第一義とし、キーはその割り当て」** に揃えるための一覧。

---

## 凡例

| 列 | 意味 |
|----|------|
| **Command name** | コマンド名（`:name` で実行するときの名前）。未定義のものは「統合時に付与する案」を記載。 |
| **Key(s)** | 割り当てキー。Browse は Browse モード、他はモード名を付記。 |
| **説明** | 動作説明。 |
| **出典** | 現状の定義場所（command = COMMAND_SPECS, main = main.rs の match）。 |

---

## 1. すでにコマンドとして存在し、キーもあるもの

| Command name | Key(s) | 説明 | 出典 |
|--------------|--------|------|------|
| quit | `q` (Browse) | アプリを終了する。 | command, main |
| help | `?` (alias、Command モードで入力) | コマンドヘルプを表示する。 | command |

※ Browse で `?` を押してもヘルプは開かない（現状は `:help` のみ）。統合時に Browse で `?` → help を割り当てるかは要検討。

---

## 2. コマンドのみ（現状キー割り当てなし）

| Command name | Key(s) | 説明 | 出典 |
|--------------|--------|------|------|
| cd | — | ディレクトリを移動: cd [path]。 | command |
| mkdir | — | ディレクトリを作成: mkdir &lt;name&gt;。 | command |
| delete | — | ファイル/ディレクトリを削除: delete [path]。 | command |
| rename | — | 選択エントリをリネーム: rename &lt;new_name&gt;。 | command |

---

## 3. キーのみ（現状コマンド未定義）→ 統合時にコマンド化する候補

| Command name (案) | Key(s) | 説明 | 出典 |
|-------------------|--------|------|------|
| open | `Enter` (Browse) | 選択したファイルを開く／ディレクトリに移動する。 | main |
| edit | `e` (Browse) | 選択ファイルをエディタで開く。 | main |
| parent | `Backspace`, `Delete` (Browse) | 親ディレクトリに移動する。 | main |
| up | `Up`, `k` (Browse) | 選択を上に移動する。 | main |
| down | `Down`, `j` (Browse) | 選択を下に移動する。 | main |
| command / cmd | `:` (Browse) | コマンドモードに入る。 | main |
| shell | `!` (Browse) | シェルモードに入る。 | main |
| filter | `/` (Browse) | フィルタモードに入る。 | main |
| create / new | `n` (Browse) | 作成モード（ファイル/ディレクトリ作成）に入る。 | main |
| toggle_status / status | `m` (Browse) | ステータスバーの詳細表示を切り替える。 | main |

---

## 4. モード専用（モード内でのみ有効なキー）

これらは「モードの UI 操作」であり、グローバルなコマンドとして名前を付けるかは任意。  
統合後も「モード専用キー」として扱う場合は、コマンド化しない選択肢あり。

### Filter モード

| 操作 | Key(s) | 説明 | 出典 |
|------|--------|------|------|
| フィルタ確定 | `Enter` | フィルタを適用して Browse に戻る。 | main |
| キャンセル | `Esc` | フィルタをやめて Browse に戻る。 | main |
| 1文字削除 | `Backspace` | フィルタ入力を1文字消す。 | main |
| 親へ移動 | `Delete` | 親ディレクトリに移動（Browse の parent 相当）。 | main |
| 上下移動 | `Up`, `Down` | リスト内の選択を移動。 | main |
| 文字入力 | 任意の文字 | フィルタ文字列に追加。 | main |

### Command モード

| 操作 | Key(s) | 説明 | 出典 |
|------|--------|------|------|
| 実行 | `Enter` | 選択/入力されたコマンドを実行。 | main |
| キャンセル | `Esc` | コマンドモードを抜ける。 | main |
| 1文字削除 | `Backspace` | 入力から1文字削除。 | main |
| 補完候補 | `Tab` / `BackTab` | 候補の次/前へ移動。 | main |
| 文字入力 | 任意の文字 | コマンド入力に追加。 | main |

### Shell モード

| 操作 | Key(s) | 説明 | 出典 |
|------|--------|------|------|
| 実行 | `Enter` | シェルコマンドを実行。 | main |
| キャンセル | `Esc` | シェルモードを抜ける。 | main |
| 1文字削除 | `Backspace` | 入力から1文字削除。 | main |
| 文字入力 | 任意の文字 | シェル入力に追加。 | main |

### Create モード

| 操作 | Key(s) | 説明 | 出典 |
|------|--------|------|------|
| 実行 | `Enter` | ファイル/ディレクトリ作成を実行。 | main |
| キャンセル | `Esc` | 作成モードを抜ける。 | main |
| 1文字削除 | `Backspace` | 入力から1文字削除。 | main |
| 文字入力 | 任意の文字 | 作成名に入力。 | main |

### 削除確認ポップアップ

| 操作 | Key(s) | 説明 | 出典 |
|------|--------|------|------|
| 削除する | `y`, `Y` | 削除を実行。 | main |
| キャンセル | `n`, `N`, `Esc`, `Enter` | 削除をやめる。 | main |

### ヘルプ/シェル結果ポップアップ

| 操作 | Key(s) | 説明 | 出典 |
|------|--------|------|------|
| 閉じる | `Esc`, `Enter` | ポップアップを閉じる。 | main |

---

## 5. 統合後の「コマンド＋キー」一覧イメージ（Browse で使うもの）

統合方針「コマンドが第一、キーは割り当て」を適用した場合の、Browse 時用の一覧。

| Command name | Key(s) | 説明 |
|--------------|--------|------|
| quit | q | 終了。 |
| help | ? | ヘルプ表示。（現状は :help のみ、? は未割り当て） |
| cd | — | ディレクトリ移動。 |
| mkdir | — | ディレクトリ作成。 |
| delete | Ctr + d | 削除。 |
| rename | Ctr + r | リネーム。 |
| open | Enter | 開く／進入。 |
| edit | e | エディタで開く。 |
| parent | Backspace, Delete | 親へ移動。 |
| up | Up, k | 選択を上へ。 |
| down | Down, j | 選択を下へ。 |
| command | : | コマンドモードへ。 |
| shell | ! | シェルモードへ。 |
| filter | / | フィルタモードへ。 |
| create | n | 作成モードへ。 |
| toggle_status | m | ステータスバー詳細の切り替え。 |

---

## 6. 実装時の参照

- コマンド定義: `src/command/mod.rs` の `COMMAND_SPECS` および `CommandId`（`src/command/types.rs`）
- Browse キー: `src/main.rs` の `Mode::Browse => match key.code { ... }`
- エディタ起動: `src/command/editor.rs` の `run(app)`

このリストを元に、1 コマンド 1 定義＋任意のキー割り当て、にリファクタする際のチェックリストとして利用できる。
