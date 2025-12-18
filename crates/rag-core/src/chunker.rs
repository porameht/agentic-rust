//! Text chunking strategies for document processing.

use common::models::{Document, DocumentChunk};

/// Text chunker for splitting documents into smaller chunks
pub struct TextChunker {
    chunk_size: usize,
    chunk_overlap: usize,
}

impl TextChunker {
    /// Create a new text chunker with specified parameters
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
        }
    }

    /// Chunk a document into smaller pieces
    pub fn chunk_document(&self, document: &Document) -> Vec<DocumentChunk> {
        self.chunk_text(&document.content)
            .into_iter()
            .enumerate()
            .map(|(index, content)| {
                let mut chunk = DocumentChunk::new(document.id, content, index);
                chunk.metadata = document.metadata.clone();
                chunk
            })
            .collect()
    }

    /// Chunk text into smaller pieces
    pub fn chunk_text(&self, text: &str) -> Vec<String> {
        if text.is_empty() {
            return Vec::new();
        }

        let mut chunks = Vec::new();
        let sentences: Vec<&str> = text
            .split(['.', '!', '?'])
            .filter(|s| !s.trim().is_empty())
            .collect();

        let mut current_chunk = String::new();

        for sentence in sentences {
            let sentence = sentence.trim();
            if sentence.is_empty() {
                continue;
            }

            let sentence_with_period = format!("{}. ", sentence);

            if current_chunk.len() + sentence_with_period.len() > self.chunk_size
                && !current_chunk.is_empty()
            {
                chunks.push(current_chunk.trim().to_string());

                // Handle overlap by keeping last part of current chunk
                if self.chunk_overlap > 0 && current_chunk.len() > self.chunk_overlap {
                    let overlap_start = current_chunk.len() - self.chunk_overlap;
                    current_chunk = current_chunk[overlap_start..].to_string();
                } else {
                    current_chunk = String::new();
                }
            }

            current_chunk.push_str(&sentence_with_period);
        }

        if !current_chunk.trim().is_empty() {
            chunks.push(current_chunk.trim().to_string());
        }

        // Fallback: if no sentence boundaries, chunk by character count
        if chunks.is_empty() && !text.is_empty() {
            let chars: Vec<char> = text.chars().collect();
            let mut i = 0;
            while i < chars.len() {
                let end = (i + self.chunk_size).min(chars.len());
                let chunk: String = chars[i..end].iter().collect();
                chunks.push(chunk);
                i += self.chunk_size - self.chunk_overlap;
            }
        }

        chunks
    }
}

impl Default for TextChunker {
    fn default() -> Self {
        Self::new(1000, 200)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text() {
        let chunker = TextChunker::new(50, 10);
        let text = "This is sentence one. This is sentence two. This is sentence three.";
        let chunks = chunker.chunk_text(text);
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_empty_text() {
        let chunker = TextChunker::default();
        let chunks = chunker.chunk_text("");
        assert!(chunks.is_empty());
    }
}
