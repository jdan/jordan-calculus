# Neovim support

This directory contains native Neovim filetype, syntax, and comment support for JordanCalculus files.

To install manually, copy these directories into your Neovim config directory:

```sh
cp -R editors/nvim/ftdetect editors/nvim/syntax editors/nvim/ftplugin ~/.config/nvim/
```

After installation, `*.jc` files use the `jordancalculus` filetype.

## Go to definition

This repository also includes a small Language Server Protocol binary:

```sh
cargo install --path . --bin jordancalculus-lsp
```

If `jordancalculus-lsp` is on your `PATH`, the ftplugin starts it automatically for `*.jc` files. Neovim's normal LSP command, `vim.lsp.buf.definition()` / `gd`, can then jump from a Katakana variable use to the nearest enclosing `J variable ッ` binder.
