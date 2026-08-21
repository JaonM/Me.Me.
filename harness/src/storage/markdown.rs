use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;

/// 开放式 Markdown 存储管理器 —— 不定义任何数据结构，纯文本读写
pub struct MarkdownStore {
    base_path: PathBuf,
}

impl MarkdownStore {
    /// 创建一个新的 Markdown 存储管理器
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// 确保目录存在
    pub fn ensure_dir(&self) -> Result<(), std::io::Error> {
        fs::create_dir_all(&self.base_path)
    }

    /// 获取文件的完整路径
    fn get_path(&self, filename: &str) -> PathBuf {
        self.base_path.join(filename)
    }

    /// 读取 Markdown 文件内容（返回原始文本）
    pub fn read(&self, filename: &str) -> Result<String, std::io::Error> {
        let path = self.get_path(filename);
        fs::read_to_string(path)
    }

    /// 写入 Markdown 文件（覆盖模式）
    pub fn write(&self, filename: &str, content: &str) -> Result<(), std::io::Error> {
        let path = self.get_path(filename);
        fs::write(path, content)
    }

    /// 追加内容到文件末尾
    pub fn append(&self, filename: &str, content: &str) -> Result<(), std::io::Error> {
        let path = self.get_path(filename);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(path)?;
        file.write_all(content.as_bytes())?;
        // 确保末尾有换行
        if !content.ends_with('\n') {
            file.write_all(b"\n")?;
        }
        Ok(())
    }

    /// 列出指定目录下的所有 Markdown 文件
    pub fn list(&self, subdir: Option<&str>) -> Result<Vec<String>, std::io::Error> {
        let target = match subdir {
            Some(dir) => self.base_path.join(dir),
            None => self.base_path.clone(),
        };
        let mut files = Vec::new();
        for entry in fs::read_dir(target)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                    files.push(filename.to_string());
                }
            }
        }
        files.sort();
        Ok(files)
    }

    /// 删除文件
    pub fn delete(&self, filename: &str) -> Result<(), std::io::Error> {
        let path = self.get_path(filename);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    /// 检查文件是否存在
    pub fn exists(&self, filename: &str) -> bool {
        self.get_path(filename).exists()
    }

    /// 获取文件大小（字节）
    pub fn size(&self, filename: &str) -> Result<u64, std::io::Error> {
        let path = self.get_path(filename);
        let metadata = fs::metadata(path)?;
        Ok(metadata.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_read_write() {
        let dir = tempdir().unwrap();
        let store = MarkdownStore::new(dir.path());
        store.ensure_dir().unwrap();

        let content = "# 标题\n\n这是正文内容。";
        store.write("test.md", content).unwrap();

        let read_content = store.read("test.md").unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_append() {
        let dir = tempdir().unwrap();
        let store = MarkdownStore::new(dir.path());
        store.ensure_dir().unwrap();

        store.write("journal.md", "2026-08-21\n---\n").unwrap();
        store.append("journal.md", "今天状态不错。\n").unwrap();
        store.append("journal.md", "下午去了咖啡馆。\n").unwrap();

        let content = store.read("journal.md").unwrap();
        assert!(content.contains("今天状态不错。"));
        assert!(content.contains("下午去了咖啡馆。"));
    }

    #[test]
    fn test_list() {
        let dir = tempdir().unwrap();
        let store = MarkdownStore::new(dir.path());
        store.ensure_dir().unwrap();

        store.write("a.md", "content a").unwrap();
        store.write("b.md", "content b").unwrap();
        store.write("c.txt", "not markdown").unwrap();

        let files = store.list(None).unwrap();
        assert_eq!(files, vec!["a.md".to_string(), "b.md".to_string()]);
    }

    #[test]
    fn test_exists_and_delete() {
        let dir = tempdir().unwrap();
        let store = MarkdownStore::new(dir.path());
        store.ensure_dir().unwrap();

        store.write("test.md", "hello").unwrap();
        assert!(store.exists("test.md"));

        store.delete("test.md").unwrap();
        assert!(!store.exists("test.md"));
    }
}