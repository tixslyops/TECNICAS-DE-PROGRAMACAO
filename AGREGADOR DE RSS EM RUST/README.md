# Agregador RSS em Rust

## 1. Sobre o projeto

O Agregador RSS em Rust é uma aplicação de linha de comando desenvolvida para cadastrar, consultar e atualizar feeds RSS e Atom.

O sistema permite acompanhar notícias e artigos de diferentes fontes em um único lugar, armazenando as informações em um banco de dados MySQL.

## 2. Integrantes

- Letícia Carvalho dos Reis

## 3. Tecnologias utilizadas

## 3. Tecnologias utilizadas

- **Rust:** linguagem de programação utilizada no desenvolvimento.
- **MySQL:** armazenamento dos feeds e artigos.
- **Tokio:** execução assíncrona.
- **Reqwest:** realização de requisições HTTP.
- **feed-rs:** interpretação de feeds RSS e Atom.
- **Clap:** criação dos comandos da aplicação.
- **Mockito:** simulação de requisições HTTP nos testes.
- **dotenvy:** leitura das configurações do arquivo `.env`.
- **Chrono:** manipulação de datas e horários.
- **Futures:** gerenciamento do processamento concorrente.
- **SHA-2:** geração de identificadores para artigos sem GUID.
- **Anyhow:** tratamento e propagação de erros.
- **URL:** validação dos endereços dos feeds.

## 4. Funcionalidades

O sistema possui as seguintes funcionalidades:

- Cadastrar feeds RSS e Atom.
- Listar feeds cadastrados.
- Remover feeds.
- Atualizar um feed específico.
- Atualizar todos os feeds cadastrados.
- Consultar todos os artigos.
- Consultar somente artigos não lidos.
- Filtrar artigos por feed.
- Pesquisar artigos pelo título ou conteúdo.
- Marcar artigos como lidos ou não lidos.
- Adicionar e remover favoritos.
- Evitar artigos duplicados.
- Armazenar informações dos feeds e artigos no MySQL.

## 5. Arquitetura do projeto

O projeto foi dividido em módulos para facilitar a organização e manutenção do código.

| Arquivo | Responsabilidade |
|---|---|
| `main.rs` | Inicialização da aplicação. |
| `lib.rs` | Organização dos módulos. |
| `cli.rs` | Interface de comandos. |
| `database.rs` | Comunicação com o MySQL. |
| `models.rs` | Estruturas de dados. |
| `parser.rs` | Interpretação de RSS e Atom. |
| `fetcher.rs` | Requisições HTTP e obtenção dos feeds. |
| `service.rs` | Regras de negócio e atualização dos feeds. |

## 6. Configuração do ambiente

### Pré-requisitos

- Rust e Cargo instalados.
- MySQL instalado.
- MySQL Workbench ou outro cliente MySQL.

### Criando o banco de dados

No MySQL Workbench, execute:

```sql
CREATE DATABASE agregador_rss
CHARACTER SET utf8mb4
COLLATE utf8mb4_unicode_ci;

USE agregador_rss;

CREATE TABLE IF NOT EXISTS feeds (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    url TEXT NOT NULL,
    last_checked DATETIME NULL,
    last_success DATETIME NULL,
    etag TEXT NULL,
    last_modified TEXT NULL,
    failure_count INT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS articles (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    feed_id BIGINT NOT NULL,
    title TEXT NOT NULL,
    author VARCHAR(255) NULL,
    link TEXT NULL,
    published DATETIME NULL,
    description LONGTEXT NULL,
    fetched_at DATETIME NOT NULL,
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    is_favorite BOOLEAN NOT NULL DEFAULT FALSE,
    guid VARCHAR(512) NOT NULL,
    CONSTRAINT fk_articles_feeds
        FOREIGN KEY (feed_id) REFERENCES feeds(id)
        ON DELETE CASCADE,
    CONSTRAINT uq_feed_guid
        UNIQUE (feed_id, guid)
);

CREATE INDEX idx_articles_feed ON articles(feed_id);
CREATE INDEX idx_articles_read ON articles(is_read);
```

### Configurando a conexão

Na pasta principal do projeto, crie um arquivo `.env`:

```env
DATABASE_URL=mysql://root:SUA_SENHA@localhost:3306/agregador_rss
```

Se o usuário root não possuir senha, utilize:

```env
DATABASE_URL=mysql://root@localhost:3306/agregador_rss
```

Substitua `SUA_SENHA` pela senha configurada no MySQL, quando aplicável.

## 7. Como executar

Abra o terminal na pasta do projeto e execute:

```bash
cargo build
```

Para iniciar a aplicação:

```bash
cargo run
```

## 8. Comandos disponíveis

### Gerenciamento de feeds

Cadastrar um feed:

```bash
cargo run -- add "BBC News" "http://feeds.bbci.co.uk/news/rss.xml"
```

Listar feeds:

```bash
cargo run -- feeds
```

Remover feed:

```bash
cargo run -- remove 1
```

### Atualização

Atualizar todos os feeds:

```bash
cargo run -- update
```

Atualizar um feed específico:

```bash
cargo run -- update-id 1
```

### Consulta de artigos

Listar todos os artigos:

```bash
cargo run -- articles
```

Listar artigos não lidos:

```bash
cargo run -- unread
```

Filtrar artigos por feed:

```bash
cargo run -- articles --feed 1
```

Pesquisar artigos:

```bash
cargo run -- search "tecnologia"
```

### Gerenciamento de leitura e favoritos

Marcar artigo como lido:

```bash
cargo run -- read 1
```

Marcar artigo como não lido:

```bash
cargo run -- unread-id 1
```

Adicionar aos favoritos:

```bash
cargo run -- favorite 1
```

Remover dos favoritos:

```bash
cargo run -- unfavorite 1
```

## 9. Decisões técnicas

### Processamento assíncrono e concorrência

Foi utilizada a biblioteca Tokio para permitir operações assíncronas.

Durante a atualização de todos os feeds, o sistema utiliza processamento concorrente com limite de cinco requisições simultâneas, evitando sobrecarregar os servidores.

### Deduplicação

A deduplicação impede que um mesmo artigo seja armazenado mais de uma vez para determinado feed.

Para isso, o sistema utiliza o identificador GUID do artigo e uma restrição de unicidade no banco de dados, considerando a combinação entre `feed_id` e `guid`.

Quando o artigo não possui identificador próprio, é gerado um identificador baseado em informações do conteúdo.

### Requisições HTTP condicionais

O sistema utiliza os cabeçalhos `ETag` e `Last-Modified`, quando disponibilizados pelo servidor.

Esses dados permitem verificar se o feed foi modificado desde a última atualização.

Quando o servidor responde com HTTP 304 (Not Modified), o sistema reconhece que o conteúdo não mudou e evita baixar novamente o feed.

### Tratamento de erros

A aplicação trata situações como:

- URLs inválidas.
- Falhas de conexão.
- Tempo limite de requisição.
- Respostas HTTP 404 e 500.
- XML inválido.
- Conteúdo que não corresponde a um feed válido.

Falhas individuais de atualização são registradas sem impedir que os demais feeds sejam processados.

## 10. Testes automatizados

Os testes foram desenvolvidos para verificar o funcionamento dos principais componentes.

Comando para executar todos os testes:

```bash
cargo test
```

Atualmente, o projeto possui 13 testes aprovados:

| Área testada | Quantidade |
|---|---:|
| Parser RSS e Atom | 5 |
| Banco de dados | 1 |
| Requisições HTTP | 6 |
| Serviço de atualização | 1 |
| **Total** | **13** |

Os testes verificam interpretação de feeds, persistência, filtros, deduplicação, erros HTTP, cabeçalhos condicionais e continuidade da atualização diante de falhas.

## 11. Limitações

- A aplicação utiliza interface de linha de comando, sem interface gráfica.
- A atualização depende da disponibilidade dos servidores que fornecem os feeds.
- A frequência de atualização é controlada pelo usuário por meio dos comandos.
- O funcionamento depende de uma conexão configurada com o banco de dados MySQL.

## 12. Uso de inteligência artificial

Ferramentas de inteligência artificial foram utilizadas como apoio durante o desenvolvimento, principalmente para auxiliar na compreensão de conceitos, identificação de erros, organização do código e elaboração de testes.

As implementações foram executadas e verificadas por meio de testes automatizados e testes manuais.

---

**Projeto acadêmico — Agregador RSS em Rust.**