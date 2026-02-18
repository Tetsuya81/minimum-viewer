# Issue #40: Shell mode runs commands in launch directory instead of current directory

## Context

Shell mode（`!`）でコマンドを実行すると、作業ディレクトリがアプリ起動時のディレクトリのままになる。ブラウザで別ディレクトリに移動した後もシェルコマンドは起動時のディレクトリで実行されてしまう。

## 原因

`src/app.rs:687` の `execute_shell_input` メソッドで `Command::new()` に `.current_dir()` が設定されていない。

```rust
// 現状（687行目）
match Command::new(&shell).arg("-lc").arg(&input).output() {
```

`self.current_dir` はブラウザの現在位置を正しく追跡しているが、シェルコマンド実行時に渡していない。

## 修正内容

### 変更ファイル: `src/app.rs`（1箇所）

687行目に `.current_dir(&self.current_dir)` を追加:

```rust
match Command::new(&shell)
    .arg("-lc")
    .arg(&input)
    .current_dir(&self.current_dir)
    .output()
{
```

## 検証方法

1. `make run` でアプリを起動
2. ブラウザで別ディレクトリに移動
3. `!` で Shell mode に入り `pwd` を実行
4. 表示されるパスが現在ブラウザで表示しているディレクトリと一致することを確認
5. `make test` でテストが通ることを確認
