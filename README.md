# minimum-viewer

Rust 製の CUI ファイルビューワー。SuperFile 風の 1 カラム UI と Helix 風の `:` コマンドモードに対応。

## 開発環境 (Nix + direnv)

```bash
direnv allow   # 初回のみ
cargo build
cargo run
```

## 操作

- **j / k** (または ↑↓): 選択移動
- **Enter**: ディレクトリに入る / ファイル選択
- **:** : コマンドモードに入る
- **q**: 終了

### コマンドモード

- `:` のあとにキーワードを入力すると候補が絞り込まれる
- **Enter**: 選択したコマンドを実行
- **Esc**: コマンドモードを抜ける
- 例: `quit`, `cd`, `help` など
