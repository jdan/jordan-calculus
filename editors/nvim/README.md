# Neovim support

This directory contains native Neovim filetype, syntax, and comment support for JordanCalculus files.

To install manually, copy these directories into your Neovim config directory:

```sh
cp -R editors/nvim/ftdetect editors/nvim/syntax editors/nvim/ftplugin ~/.config/nvim/
```

After installation, `*.jc` files use the `jordancalculus` filetype.
