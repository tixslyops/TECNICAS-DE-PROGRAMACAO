use crate::models::{Artigo, Feed};
use rusqlite::{params, Connection, Result};
use chrono::{DateTime, Utc};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Database { conn };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS feeds (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL UNIQUE,
                etag TEXT,
                last_modified TEXT,
                last_checked TEXT,
                failure_count INTEGER DEFAULT 0
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS artigos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                feed_id INTEGER NOT NULL,
                guid_hash TEXT NOT NULL UNIQUE,
                titulo TEXT NOT NULL,
                autor TEXT,
                link TEXT NOT NULL,
                descricao TEXT,
                data_publicacao TEXT,
                data_obtencao TEXT NOT NULL,
                lido INTEGER DEFAULT 0,
                favorito INTEGER DEFAULT 0,
                FOREIGN KEY(feed_id) REFERENCES feeds(id) ON DELETE CASCADE
            )",
            [],
        )?;

        Ok(())
    }

    pub fn adicionar_feed(&self, url: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO feeds (url, failure_count) VALUES (?1, 0)",
            params![url],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn remover_feed(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM feeds WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn listar_feeds(&self) -> Result<Vec<Feed>> {
        let mut stmt = self.conn.prepare("SELECT id, url, etag, last_modified, last_checked, failure_count FROM feeds")?;
        let feed_iter = stmt.query_map([], |row| {
            let last_checked_str: Option<String> = row.get(4)?;
            Ok(Feed {
                id: row.get(0)?,
                url: row.get(1)?,
                etag: row.get(2)?,
                last_modified: row.get(3)?,
                last_checked: last_checked_str.and_then(|s| s.parse::<DateTime<Utc>>().ok()),
                failure_count: row.get(5)?,
            })
        })?;

        let mut feeds = Vec::new();
        for feed in feed_iter {
            feeds.push(feed?);
        }
        Ok(feeds)
    }

    pub fn salvar_artigo(&self, artigo: &Artigo) -> Result<bool> {
        let data_pub = artigo.data_publicacao.map(|d| d.to_rfc3339());
        let data_obt = artigo.data_obtencao.to_rfc3339();

        let result = self.conn.execute(
            "INSERT OR IGNORE INTO artigos 
            (feed_id, guid_hash, titulo, autor, link, descricao, data_publicacao, data_obtencao, lido, favorito)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                artigo.feed_id,
                artigo.guid_hash,
                artigo.titulo,
                artigo.autor,
                artigo.link,
                artigo.descricao,
                data_pub,
                data_obt,
                artigo.lido as i32,
                artigo.favorito as i32,
            ],
        );

        match result {
            Ok(rows) => Ok(rows > 0),
            Err(e) => Err(e),
        }
    }

    pub fn atualizar_status_feed(&self, id: i64, etag: Option<String>, last_modified: Option<String>, falha: bool) -> Result<()> {
        let agora = Utc::now().to_rfc3339();
        if falha {
            self.conn.execute(
                "UPDATE feeds SET last_checked = ?1, failure_count = failure_count + 1 WHERE id = ?2",
                params![agora, id],
            )?;
        } else {
            self.conn.execute(
                "UPDATE feeds SET etag = ?1, last_modified = ?2, last_checked = ?3, failure_count = 0 WHERE id = ?4",
                params![etag, last_modified, agora, id],
            )?;
        }
        Ok(())
    }

    pub fn listar_artigos(&self, apenas_nao_lidos: bool, filtro_texto: Option<&str>) -> Result<Vec<Artigo>> {
        let mut query = String::from(
            "SELECT id, feed_id, guid_hash, titulo, autor, link, descricao, data_publicacao, data_obtencao, lido, favorito FROM artigos WHERE 1=1"
        );

        if apenas_nao_lidos {
            query.push_str(" AND lido = 0");
        }

        if let Some(texto) = filtro_texto {
            query.push_str(&format!(" AND (titulo LIKE '%{}%' OR descricao LIKE '%{}%')", texto, texto));
        }

        query.push_str(" ORDER BY data_publicacao DESC");

        let mut stmt = self.conn.prepare(&query)?;
        let artigo_iter = stmt.query_map([], |row| {
            let data_pub_str: Option<String> = row.get(7)?;
            let data_obt_str: String = row.get(8)?;

            Ok(Artigo {
                id: row.get(0)?,
                feed_id: row.get(1)?,
                guid_hash: row.get(2)?,
                titulo: row.get(3)?,
                autor: row.get(4)?,
                link: row.get(5)?,
                descricao: row.get(6)?,
                data_publicacao: data_pub_str.and_then(|s| s.parse::<DateTime<Utc>>().ok()),
                data_obtencao: data_obt_str.parse::<DateTime<Utc>>().unwrap_or_else(|_| Utc::now()),
                lido: row.get::<_, i32>(9)? != 0,
                favorito: row.get::<_, i32>(10)? != 0,
            })
        })?;

        let mut artigos = Vec::new();
        for artigo in artigo_iter {
            artigos.push(artigo?);
        }
        Ok(artigos)
    }

    pub fn marcar_lido(&self, id: i64, lido: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE artigos SET lido = ?1 WHERE id = ?2",
            params![lido as i32, id],
        )?;
        Ok(())
    }
}