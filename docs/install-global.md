# どこからでもコマンドで起動する手順

`minimum-viewer` をローカルの任意のディレクトリからコマンドで起動できるようにする方法です。起動コマンドは **`mmv`** です。

---

## 方法1: cargo install（推奨）

Rust の標準的な「グローバルインストール」です。ビルドしたバイナリが `~/.cargo/bin/` に入り、そこが PATH に入っていればどこからでも `mmv` で起動できます。

### 手順

1. **プロジェクトのルートに移動**
   ```bash
   cd /Users/tetsuya81/dev/MyGithub/minimum-viewer
   ```

2. **インストール実行**
   ```bash
   cargo install --path .
   ```
   - リリースビルドでコンパイルされ、`~/.cargo/bin/mmv` に配置されます。

3. **PATH の確認**
   - Rust (rustup) を入れた状態なら、通常 `~/.cargo/bin` はすでに PATH に入っています。
   - 未設定の場合はシェル設定（例: `~/.zshrc`）に次を追加：
     ```bash
     export PATH="$HOME/.cargo/bin:$PATH"
     ```
   - 反映: `source ~/.zshrc` またはターミナルを開き直す。

4. **動作確認**
   ```bash
   cd ~
   mmv
   ```
   任意のディレクトリで上記が動けば完了です。

### 更新するとき

ソースを変更したあと、再度インストールし直します。

```bash
cd /Users/tetsuya81/dev/MyGithub/minimum-viewer
cargo install --path .
```

---

## 方法2: シンボリックリンクを PATH のディレクトリに置く

「ビルド成果物のバイナリ」を直接 PATH の通った場所から参照する方法です。

1. **リリースビルド**
   ```bash
   cd /Users/tetsuya81/dev/MyGithub/minimum-viewer
   cargo build --release
   ```
   バイナリ: `target/release/mmv`

2. **PATH のディレクトリにリンクを張る**
   - 置き場所は好みでよい。よく使われるのは次のどちらかです。
   - **`~/bin`** の例:
     ```bash
     mkdir -p ~/bin
     ln -sf /Users/tetsuya81/dev/MyGithub/minimum-viewer/target/release/mmv ~/bin/mmv
     export PATH="$HOME/bin:$PATH"   # 未設定なら ~/.zshrc に追加
     ```
   - **`~/.local/bin`** の例（Linux では XDG の慣習でこちらを使うことも多い）:
     ```bash
     mkdir -p ~/.local/bin
     ln -sf /Users/tetsuya81/dev/MyGithub/minimum-viewer/target/release/mmv ~/.local/bin/mmv
     export PATH="$HOME/.local/bin:$PATH"   # 未設定なら ~/.zshrc に追加
     ```
   - いずれも `~/.zshrc` に書いておくと永続的です。

3. **実行**
   ```bash
   mmv
   ```

プロジェクトを更新したときは、再度 `cargo build --release` を実行すれば、同じリンクから新しいバイナリが使われます。

---

## 方法3: エイリアスでプロジェクトのバイナリを指定する

インストールはせず、常に「このプロジェクトの release バイナリ」を叩く方法です。

`~/.zshrc` に追加:

```bash
alias mmv='/Users/tetsuya81/dev/MyGithub/minimum-viewer/target/release/mmv'
```

その後、`cargo build --release` を一度実行し、`source ~/.zshrc` すれば、どこからでも `mmv` で起動できます。更新時はプロジェクトで `cargo build --release` を実行するだけです。

---

## まとめ

| 方法 | メリット | 向いている人 |
|------|----------|----------------|
| **cargo install** | Rust の一般的なやり方で、バージョン管理しやすい | まずはこれでよい |
| **シンボリックリンク** | ビルド先を自分で選べる | `~/bin` で統一したい人 |
| **エイリアス** | インストール不要で手軽 | 一時的・開発中だけ使いたい人 |

通常は **方法1（cargo install）** を使うのがおすすめです。

---

## 開発しながら自分で使うときのよくあるパターン

Rust の CUI/CLI を開発しながら、自分で実際に使う場合によく取られる方法です。

### パターンA: cargo install を更新のたびに叩く

- **やり方**: 変更のたびに `cargo install --path .`（必要なら `--force`）を実行する。
- **特徴**: Rust の公式ドキュメントや「CLI の配布」の説明でよく紹介される標準的なやり方。`~/.cargo/bin` に常に「使う用」のバイナリが入る。
- **向いている人**: インストール手順を一本にしたい人、crates.io や `cargo install <crate>` の感覚で扱いたい人。

### パターンB: シンボリックリンク + cargo build --release（開発中はこちらの人が多い）

- **やり方**: 初回だけ `cargo build --release` してから、`target/release/mmv` を `~/bin` など PATH の通った場所にシンボリックリンク。以降はコードを変えたら **`cargo build --release` だけ**で、同じ `mmv` コマンドが常に最新ビルドを指す。
- **特徴**: 「インストール」というステップがほぼ不要。ビルド＝更新なので、開発中のイテレーションが速い。Rust コミュニティでも自分用ツールを開発しながら使うときによく使われる。
- **向いている人**: こまめに直しては試したい人、`cargo install` を毎回打ちたくない人。

### パターンC: 開発中は cargo run、たまに使うときだけインストール

- **やり方**: 普段はプロジェクト直下で `cargo run`。他のディレクトリから「ツールとして」使うときだけ `cargo install --path .` する。
- **特徴**: 設定が少ない。ただし「今のコード」をどこからでも使うには、そのたびに install が必要。

### まとめ

| やり方 | 更新の手間 | よくある使い分け |
|--------|------------|------------------|
| cargo install を都度 | 毎回 `cargo install --path .` | 配布・説明と揃えたいとき |
| シンボリックリンク | `cargo build --release` だけ | **開発しながら毎日使う**とき |
| cargo run のみ | プロジェクト内で run | まず動かして確認したいとき |

**開発しながら自分で利用する**ことが主なら、**方法2（シンボリックリンク）** で `target/release/mmv` を指しておき、更新は `cargo build --release` にすると楽なことが多いです。
