# PR28 — UBL ID (People · LLM · Apps)

Status: draft
Generated: 2025-12-26T00:37:03.542211Z

Este pacote entrega **API + schema + stubs** para identidade unificada:
- **Pessoas**: passkey (WebAuthn) — *sem senha*.
- **LLM Agents**: Ed25519 + certificado de agente (ASC).
- **Apps**: credencial de servidor (Ed25519 ou mTLS), com escopos.

---

## Done if… (checagem objetiva)

1. `POST /id/register/begin|finish` registra passkey com `auth_user` + `auth_credential` criados.
2. `POST /id/login/begin|finish` autentica e emite sessão (cookie httpOnly).
3. `POST /id/agents` cria **subject** tipo `llm` ou `app` com **pubkey** registrada.
4. `POST /id/agents/{sid}/asc` emite **Agent Signing Certificate** (ASC) com *escopos e TTL*.
5. `POST /id/agents/{sid}/rotate` efetiva **rotação de chave** (versão + revogação anterior).
6. `POST /id/sessions/ict/begin|finish` abre **ICTE** com `max_delta`, `scope`, `ttl`.
7. `GET /id/whoami` retorna *subject + roles + sessions ativas*.
8. **Tests** cobrem: reuse de challenge, origin inválida, counter regressivo, asc expirada, rotação, e abuso de escopo.

---

## Test Plan (pega erro de verdade)

- **Challenge reuse** (409), **origin spoof** (403), **UV ausente** (dev=preferred; admin=required em PR29).
- **Counter regressivo** na assertion (400) e persistência de `sign_count`.
- **ASC expirada** → `401 Unauthorized` (assinatura ok, certificado inválido).
- **Rotação**: assinatura com chave antiga após revogação → `401`.
- **ICTE**: uso fora de janela ou `delta` > `max_delta` → `PactViolation` no commit.
