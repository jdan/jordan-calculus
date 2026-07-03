vim.bo.commentstring = "え %s"
vim.bo.comments = ":え"

local function set_direct_definition_keymap(bufnr)
  if vim.api.nvim_buf_is_valid(bufnr) then
    vim.keymap.set("n", "gd", vim.lsp.buf.definition, {
      buffer = bufnr,
      desc = "JordanCalculus go to definition",
    })
  end
end

-- LazyVim/Snacks may route `gd` through a picker that reports "No results"
-- when the definition result is the current token. JordanCalculus definitions are
-- single-token binders, so use Neovim's direct LSP jump for this filetype.
local bufnr = vim.api.nvim_get_current_buf()
set_direct_definition_keymap(bufnr)
for _, delay in ipairs({ 50, 100, 250, 500, 1000 }) do
  vim.defer_fn(function()
    set_direct_definition_keymap(bufnr)
  end, delay)
end

vim.api.nvim_create_autocmd("LspAttach", {
  buffer = bufnr,
  callback = function(args)
    local client = vim.lsp.get_client_by_id(args.data.client_id)
    if client and client.name == "jordancalculus-lsp" then
      set_direct_definition_keymap(args.buf)
      for _, delay in ipairs({ 50, 100, 250, 500, 1000 }) do
        vim.defer_fn(function()
          set_direct_definition_keymap(args.buf)
        end, delay)
      end
    end
  end,
})

if vim.fn.executable("jordancalculus-lsp") == 1 then
  vim.lsp.start({
    name = "jordancalculus-lsp",
    cmd = { "jordancalculus-lsp" },
    root_dir = vim.fs.root(0, { ".git", "Cargo.toml" }) or vim.fn.getcwd(),
  })
end
