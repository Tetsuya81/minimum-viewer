# minimum-viewer

Rust 製の CUI ファイルビューワー。SuperFile 風の 1 カラム UI と Helix 風の `:` コマンドモードに対応。

UI改善時の用語統一は [UI名称ガイド](docs/ui-naming.md) を参照してください。

## 開発環境 (Nix + direnv)

```bash
direnv allow   # 初回のみ
cargo build
cargo run
```

## テスト / ビルド実行ルール

このリポジトリでは、環境差分を避けるため Nix 経由で実行します。

```bash
make test   # nix develop -c cargo test
make build  # nix develop -c cargo build
make run    # nix develop -c cargo run
```

## 操作

- **j / k** (または ↑↓): 選択移動
- **Enter**: ディレクトリに入る / ファイル選択
- **Delete / Backspace**: 親ディレクトリに移動（親がない場合はステータス表示）
- **/**: ファイル一覧フィルタモードに入る
- **:** : コマンドモードに入る
- **!** : シェル1行実行モードに入る
- **e**: 選択中のファイル/ディレクトリを `$EDITOR` で開く
- **m**: Browse の `Status Bar` を展開/折りたたみ（詳細メタデータの表示切替）
- **q**: 終了

### フィルタモード

- `/` のあとに文字を入力すると、ファイル/ディレクトリ名で部分一致フィルタ
- 大文字小文字は無視される
- `..` は常に表示される
- **Enter**: フィルタを保持したまま Browse に戻る
- **Esc**: フィルタをクリアして Browse に戻る
- ディレクトリ移動（`Enter` / `Delete` / `Backspace` / `cd`）時はフィルタが自動解除される

### コマンドモード

- `:` のあとにキーワードを入力すると候補が絞り込まれる
- **Tab / Shift+Tab**: 候補選択（前後循環）
- **Enter**: 選択したコマンドを実行
- **Esc**: コマンドモードを抜ける
- 利用可能コマンド: `quit` (`q`), `cd`, `mkdir`, `delete`, `rename`, `help` (`?`)
- `cd` は `cd`（選択ディレクトリへ移動）または `cd <path>`（絶対/相対/`~`対応）
- `mkdir` は `mkdir <directory_name>` のみ対応（単一トークン）
- `delete` は `delete [path]`。引数なし時は選択中エントリを対象にする
  - ファイル削除は即時実行、ディレクトリ削除は確認ポップアップで `y/N`
- `rename` は `rename <new_name>`。選択中エントリ名を同一ディレクトリ内で変更

### シェルモード

- `!` のあとに1行コマンドを入力
- **Enter**: `$SHELL -lc` で実行（`SHELL` 未設定時は `/bin/sh -lc`）
- 実行後、結果はポップアップ表示（**Enter/Esc** で閉じる）

## 環境変数

- `MINIMUM_VIEWER_CWD`: アプリ内の現在ディレクトリ
- 起動時とディレクトリ移動時（`Enter` / `cd`）に更新される
- 親シェルには伝播せず、アプリプロセス内でのみ有効
- `MINIMUM_VIEWER_CONFIG`: 設定ファイルのパスを明示する（未指定時は XDG パスを利用）
- `MINIMUM_VIEWER_LAST_DIR`: `cd_on_quit` で使うコマンドファイルのパスを明示する
  - 未指定時は `${XDG_STATE_HOME:-$HOME/.local/state}/mmv/lastdir` を利用
- `EDITOR`: `e` キーバインドで利用するエディタ。未設定時はエラー表示

`XDG_CONFIG_HOME` は `config.toml` の配置にのみ使い、`lastdir` には使いません。

## `cd_on_quit` を有効化する場合の shell wrapper

`cd_on_quit = true` のとき、`mmv` は終了時に `cd -- '...'` コマンドを `lastdir` ファイルへ書き込みます。  
wrapper は `lastdir` ファイルを `. (source)` してから削除します。`~/.zshrc` か `~/.bashrc` に次を追加してください。

```bash
mmv() {
  local lastdir="${MINIMUM_VIEWER_LAST_DIR:-${XDG_STATE_HOME:-$HOME/.local/state}/mmv/lastdir}"
  command mmv "$@"
  local exit_code=$?
  if [ $exit_code -eq 0 ] && [ -f "$lastdir" ]; then
    . "$lastdir"
    rm -f "$lastdir"
  fi
  return $exit_code
}
```

設定ファイル例（`$XDG_CONFIG_HOME/mmv/config.toml` または `~/.config/mmv/config.toml`）:

```toml
cd_on_quit = true
```
