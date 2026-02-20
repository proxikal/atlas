# Atlas LSP - Emacs Setup

Configure the Atlas Language Server with Emacs using lsp-mode or eglot.

---

## Using lsp-mode

Add to your `init.el`:

```elisp
(use-package lsp-mode
  :commands lsp
  :config
  (lsp-register-client
   (make-lsp-client
    :new-connection (lsp-stdio-connection '("atlas" "lsp"))
    :major-modes '(atlas-mode)
    :server-id 'atlas-lsp)))

(add-hook 'atlas-mode-hook #'lsp)
```

---

## Using eglot

```elisp
(use-package eglot
  :config
  (add-to-list 'eglot-server-programs
               '(atlas-mode . ("atlas" "lsp"))))

(add-hook 'atlas-mode-hook #'eglot-ensure)
```

---

## Key Bindings

With lsp-mode:
```elisp
(define-key lsp-mode-map (kbd "C-c l") lsp-command-map)
```

Common bindings:
- `M-.` - Go to definition
- `M-?` - Find references
- `C-c C-f` - Format buffer

---

## Troubleshooting

View LSP events:
```
M-x lsp-workspace-show-log
```

Restart server:
```
M-x lsp-workspace-restart
```

---

## More Information

- [LSP Features](../lsp-features.md)
- [Troubleshooting](../lsp-troubleshooting.md)
