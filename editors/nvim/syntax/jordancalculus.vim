if exists("b:current_syntax")
  finish
endif

syntax case match

" Entire-line comments. Leading whitespace is allowed.
syntax match jordanCalculusComment /^\s*え.*$/

" Top-level definition macro.
syntax match jordanCalculusDefinition /^\s*上げる\>/
syntax match jordanCalculusDefinitionParticle /は/

" Core JordanCalculus notation.
syntax match jordanCalculusLambda /J/
syntax match jordanCalculusDot /ッ/
syntax match jordanCalculusApplication /足す/
syntax match jordanCalculusParen /[「」]/

" One or more Katakana characters, excluding small tsu ッ because it marks abstraction.
syntax match jordanCalculusVariable /[ァ-ヂツ-ヿㇰ-ㇿ]\+/

highlight default link jordanCalculusComment Comment
highlight default link jordanCalculusDefinition Keyword
highlight default link jordanCalculusDefinitionParticle Keyword
highlight default link jordanCalculusLambda Keyword
highlight default link jordanCalculusDot Keyword
highlight default link jordanCalculusApplication Operator
highlight default link jordanCalculusParen Delimiter
highlight default link jordanCalculusVariable Identifier

let b:current_syntax = "jordancalculus"
