# Issue案: Ubuntu へ SSH しているとき `yank` が `clipboard unavailable` になる

## 背景

`yank` は選択中エントリのパスをクリップボードへコピーするコマンドだが、
Linux では現在 `wl-copy` / `xclip` / `xsel` の外部ツールだけに依存している。

そのため、SSH 先の Ubuntu 上で minimum-viewer を使うと、
リモート側に GUI クリップボードが存在しない環境では `yank: clipboard unavailable` になって失敗する。

## 目的

SSH 経由で Ubuntu 上の minimum-viewer を利用している場合でも、
ローカル端末のクリップボードへパスをコピーできるようにする。

## 提案仕様

- `yank` の既存インターフェースは維持する
  - コマンド名: `yank`
  - エイリアス: `y`
  - 引数仕様: `yank [path]`
- 既存の外部クリップボードツールを引き続き優先する
  - macOS: `pbcopy`
  - Linux: `wl-copy` / `xclip` / `xsel`
- Linux で上記ツールが使えず、かつ SSH セッション中と判定できる場合は `OSC 52` をフォールバックとして使う
- `OSC 52` は標準出力へエスケープシーケンスを書き込んで、接続元ターミナルのクリップボードへ転送する
- 端末や `tmux` 設定で `OSC 52` が拒否された場合は、既存どおり短い失敗メッセージを表示する

## 実装ポイント

- `src/command/yank.rs`
  - `copy_to_clipboard` を整理して、外部ツール失敗後の SSH 向け `OSC 52` フォールバックを追加する
  - SSH 判定は `SSH_CONNECTION` を第一候補にし、必要に応じて `SSH_CLIENT` / `SSH_TTY` も見る
  - `OSC 52` 用にテキストを Base64 化し、`ESC ] 52 ; c ; ... BEL` を stdout に送る処理を追加する
- `Cargo.toml`
  - Base64 エンコード用に軽量依存を追加するか、最小の自前実装を置く
- `README.md`
  - `yank` が SSH 上では `OSC 52` を使うこと
  - 端末や `tmux` の設定次第で無効な場合があること
  - Ubuntu 側に `xclip` 等がなくても SSH 経由ならコピー可能になること

## 受け入れ条件

1. Ubuntu に SSH して minimum-viewer を起動し、`yank` 実行でローカル側クリップボードにパスが入る
2. macOS ローカルの `pbcopy` 動作は壊れない
3. Linux ローカルで `wl-copy` / `xclip` / `xsel` が使える場合は従来どおりそちらが優先される
4. SSH でない Linux 環境で外部ツールが無い場合は、従来どおり失敗する
5. `OSC 52` が使えない端末でも無言で失敗せず、ステータスメッセージに失敗が出る

## テスト観点

- 単体テスト
  - 外部ツールが unavailable で SSH 環境なら `OSC 52` 経路を選ぶ
  - 外部ツールが unavailable で SSH 環境でなければ失敗する
  - `OSC 52` シーケンスの Base64 エンコード結果が期待どおりになる
- 手動確認
  - macOS ローカルで `yank`
  - Ubuntu over SSH で `yank`
  - `tmux` 配下での挙動確認

## 補足

今回は最小スコープとして、`tmux` 専用の追加エスケープ対応や細かい端末互換性対応までは含めない。
まずは素の SSH ターミナルで `yank` が実用になることを優先する。
