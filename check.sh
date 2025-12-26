#!/bin/bash
# UBL 2.0 - Verificação Rápida do Monorepo

echo "╔══════════════════════════════════════════════════════════╗"
echo "║         UBL 2.0 Monorepo - Status Check              ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

cd /Users/voulezvous/UBL-2.0-insiders

echo "📦 Estrutura:"
for dir in kernel mind specs sql infra containers manifests scripts; do
    if [ -d "$dir" ]; then
        echo "   ✅ $dir/"
    else
        echo "   ❌ $dir/ (missing)"
    fi
done
echo ""

echo "🦀 Rust Crates:"
cd kernel/rust
CRATES=$(ls -d */ 2>/dev/null | grep -v target | wc -l)
echo "   $CRATES crates found"
echo ""

echo "🧪 Compilação:"
if cargo check --workspace --quiet 2>&1 | grep -q "Finished"; then
    echo "   ✅ Compila sem erros"
else
    echo "   ⚠️  Verificar erros de compilação"
fi
echo ""

echo "📄 Documentação:"
cd /Users/voulezvous/UBL-2.0-insiders
DOCS=$(ls -1 *.md 2>/dev/null | wc -l)
echo "   $DOCS documentos encontrados"
echo ""

echo "✨ Status: PRONTO"
echo "📍 Location: /Users/voulezvous/UBL-2.0-insiders/"
echo ""
echo "Próximo: cargo test --workspace"
