# prox — Painel local de portas e processos (TUI)

**Data:** 2026-06-10
**Status:** Design aprovado
**Stack:** Rust (TUI), Linux apenas no v1

## Problema

Todo dev Linux esbarra no clássico `Error: port 3000 already in use` e precisa
parar pra rodar `lsof -i :3000`, ler a saída, achar o PID e matar na mão. As
ferramentas atuais (`lsof`, `ss`, `netstat`) são de linha de comando, com saída
crua e sem interatividade. Não existe um jeito rápido e visual de ver o que está
escutando em cada porta e matar o processo na hora.

## Objetivo

Um binário único de TUI que, ao abrir, mostra ao vivo todas as portas em escuta
no sistema com seus processos donos, permite filtrar/buscar e matar o processo
selecionado com uma tecla. Foco na dor diária do desenvolvedor local.

Projeto open source pensado também como peça de portfólio (binário instalável,
README com GIF do TUI em ação).

## Escopo do v1

Incluído:
- Listar todas as portas TCP/UDP em escuta (IPv4 e IPv6) com o processo dono.
- Atualização automática (~1s) e recarga manual.
- Navegação por teclado, filtro/busca ao vivo, ordenação.
- Matar processo selecionado (SIGTERM → SIGKILL), com confirmação.
- Tratamento claro de erro de permissão (matar processo de outro usuário).
- Somente Linux.

Fora do v1 (futuro):
- Gerenciar serviços do projeto (start/stop de db/api/front, ler config do
  projeto). É outro projeto em tamanho — fica para o v2.
- Suporte a macOS / Windows.

## Decisão técnica: coleta de dados

Mapear porta → processo no Linux será feito lendo `/proc` diretamente
(abordagem escolhida entre: A) ler `/proc`, B) chamar `ss`/`lsof`, C) crate
`netstat2`).

- Parsear sockets em escuta de `/proc/net/{tcp,tcp6,udp,udp6}` (porta + inode).
- Cruzar o inode do socket com os symlinks `/proc/<pid>/fd/*` (`socket:[inode]`)
  para descobrir o PID dono.
- Coletar metadados do processo: nome (`/proc/<pid>/comm`), linha de comando
  (`/proc/<pid>/cmdline`), usuário (dono do `/proc/<pid>`).

Implementado sobre a crate `procfs` para não reinventar o parsing. Sem
dependência de binários externos em runtime — robusto e bom de mostrar.

## Arquitetura (módulos)

Cada módulo tem um propósito único e é testável isoladamente. Fluxo:
`collector → model → app → ui`; efeitos colaterais isolados em `actions`.

- **`collector`** — única parte que fala com `/proc`. Tira um snapshot do estado
  e devolve structs puras (porta, protocolo, PID, nome, cmdline, usuário). Sem UI.
- **`model`** — tipos de dados (`PortEntry`, `Process`) e a lógica pura de filtro
  e ordenação. Fácil de testar.
- **`app`** — estado da aplicação: lista atual, texto do filtro, item
  selecionado, modo (navegando / confirmando kill), critério de ordenação,
  intervalo de refresh, mensagens de status. Recebe eventos e atualiza o estado.
  Não desenha nada e não lê `/proc` direto — só orquestra.
- **`ui`** — função pura `&App -> tela` desenhada com `ratatui`. Sem lógica de
  negócio.
- **`actions`** — efeitos colaterais isolados: matar processo (SIGTERM, depois
  SIGKILL se necessário).
- **`main`** — junta tudo: loop de eventos (input de teclado + tick de refresh),
  chama `collector`, atualiza `app`, renderiza `ui`.

## UI / UX

Tela única com três zonas: tabela de portas, barra de filtro, barra de ajuda.

```
┌─ prox ───────────────────────────── 12 portas ─┐
│ PORTA  PROTO  PID    PROCESSO     USUÁRIO  CMD   │
│ 3000   TCP    48213  node         elias    npm…  │  ← selecionada
│ 5432   TCP    1190   postgres     postgres /usr… │
│ 6379   TCP    922    redis-server redis    …     │
├──────────────────────────────────────────────────┤
│ /node_           (filtro)        ↑↓ mover  / busca│
│ k matar   r recarregar   q sair                   │
└──────────────────────────────────────────────────┘
```

Teclas:
- `↑/↓` ou `j/k` — navegar.
- `/` — entrar no modo filtro (filtra ao vivo por porta / processo / PID);
  `Esc` limpa/sai do filtro.
- `k` ou `Del` — matar o processo selecionado → abre confirmação (`s/n`);
  SIGTERM primeiro, SIGKILL se não morrer.
- `r` — recarregar imediatamente.
- `s` — alternar ordenação (porta / processo / PID).
- `q` — sair.

Permissão: matar processo de outro usuário pode falhar por falta de privilégio.
Nesse caso, exibir aviso claro na barra de status — nunca quebrar a aplicação.

## Stack e dependências

- `ratatui` — renderização da TUI.
- `crossterm` — controle de terminal e captura de input.
- `procfs` — leitura de sockets e processos via `/proc`.
- `nix` — envio de sinais (kill).
- Sem async: loop simples com timeout no input é suficiente.

## Testes

- `model`: filtro e ordenação testados com dados de exemplo.
- `collector`: a transformação de dados crus → structs testada com fixtures
  (sem depender do `/proc` real da máquina de teste).
- `app`: testado simulando sequências de eventos (digitar filtro → lista muda;
  apertar kill → entra em modo confirmação; confirmar → chama `actions`).
- `ui` e leitura real de `/proc`: fora dos testes unitários, validados
  manualmente.

## Entrega

- Binário único; instalável via `cargo install` e release no GitHub.
- README com descrição, instalação e GIF do TUI rodando.
